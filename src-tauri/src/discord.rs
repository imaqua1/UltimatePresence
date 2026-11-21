//! Minimal Discord RPC client over local IPC (named pipe on Windows, Unix socket
//! on macOS/Linux). Keeps reconnecting while Discord is closed and re-sends the
//! current activity whenever a connection comes up.

use crate::activity::Activity;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::{mpsc, watch, Notify};
use tokio::time::{sleep, sleep_until, timeout, Instant};

const OP_HANDSHAKE: u32 = 0;
const OP_FRAME: u32 = 1;
const OP_CLOSE: u32 = 2;
const OP_PING: u32 = 3;
const OP_PONG: u32 = 4;

const RETRY_EVERY: Duration = Duration::from_secs(5);
/// Coalesce bursts of updates; Discord rate-limits activity changes anyway.
const MIN_SEND_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, PartialEq)]
pub enum DiscordStatus {
    Disconnected,
    Connected { user: String, avatar: Option<String> },
    /// Discord refused the handshake (usually an invalid Application ID).
    Rejected(String),
}

pub enum DiscordEvent {
    Status(DiscordStatus),
    Log(&'static str, String),
}

pub struct DiscordHandle {
    activity_tx: watch::Sender<Option<Activity>>,
    client_id_tx: watch::Sender<String>,
    shutdown_tx: watch::Sender<bool>,
    done: Arc<Notify>,
}

/// The not-yet-started background task. Split from the handle so the app can
/// register state before the first event fires.
pub struct DiscordWorker {
    activity_rx: watch::Receiver<Option<Activity>>,
    client_id_rx: watch::Receiver<String>,
    shutdown_rx: watch::Receiver<bool>,
    done: Arc<Notify>,
}

pub fn new(client_id: String) -> (DiscordHandle, DiscordWorker) {
    let (activity_tx, activity_rx) = watch::channel(None);
    let (client_id_tx, client_id_rx) = watch::channel(client_id);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let done = Arc::new(Notify::new());
    (
        DiscordHandle { activity_tx, client_id_tx, shutdown_tx, done: done.clone() },
        DiscordWorker { activity_rx, client_id_rx, shutdown_rx, done },
    )
}

impl DiscordHandle {
    pub fn set_activity(&self, activity: Option<Activity>) {
        self.activity_tx.send_replace(activity);
    }

    pub fn set_client_id(&self, id: String) {
        self.client_id_tx.send_replace(id);
    }

    /// Clears the activity on Discord (if connected) and stops the worker.
    pub async fn shutdown(&self, max_wait: Duration) {
        self.shutdown_tx.send_replace(true);
        let _ = timeout(max_wait, self.done.notified()).await;
    }
}

trait IpcStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> IpcStream for T {}

#[cfg(windows)]
async fn open_ipc() -> std::io::Result<Box<dyn IpcStream>> {
    use tokio::net::windows::named_pipe::ClientOptions;
    let mut last = None;
    for i in 0..10 {
        match ClientOptions::new().open(format!(r"\\.\pipe\discord-ipc-{i}")) {
            Ok(pipe) => return Ok(Box::new(pipe)),
            Err(e) => last = Some(e),
        }
    }
    Err(last.unwrap())
}

#[cfg(unix)]
async fn open_ipc() -> std::io::Result<Box<dyn IpcStream>> {
    let base = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"]
        .iter()
        .find_map(|k| std::env::var(k).ok())
        .unwrap_or_else(|| "/tmp".into());
    for i in 0..10 {
        let path = std::path::Path::new(&base).join(format!("discord-ipc-{i}"));
        if let Ok(s) = tokio::net::UnixStream::connect(&path).await {
            return Ok(Box::new(s));
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "no Discord IPC socket"))
}

async fn write_frame<W: AsyncWrite + Unpin>(w: &mut W, op: u32, payload: &Value) -> std::io::Result<()> {
    let body = serde_json::to_vec(payload)?;
    let mut buf = Vec::with_capacity(8 + body.len());
    buf.extend_from_slice(&op.to_le_bytes());
    buf.extend_from_slice(&(body.len() as u32).to_le_bytes());
    buf.extend_from_slice(&body);
    w.write_all(&buf).await?;
    w.flush().await
}

async fn read_frame<R: AsyncRead + Unpin>(r: &mut R) -> std::io::Result<(u32, Value)> {
    let mut header = [0u8; 8];
    r.read_exact(&mut header).await?;
    let op = u32::from_le_bytes(header[0..4].try_into().unwrap());
    let len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
    if len > 1 << 20 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "frame too large"));
    }
    let mut body = vec![0u8; len];
    r.read_exact(&mut body).await?;
    Ok((op, serde_json::from_slice(&body).unwrap_or(Value::Null)))
}

enum ConnectError {
    NotRunning,
    Rejected(String),
    Failed(String),
}

/// Your Discord profile picture (or Discord's default one) from the READY event.
fn avatar_url(user: &Value) -> Option<String> {
    let id = user["id"].as_str()?;
    match user["avatar"].as_str() {
        Some(hash) if !hash.is_empty() => Some(format!("https://cdn.discordapp.com/avatars/{id}/{hash}.png?size=256")),
        _ => {
            let index = id.parse::<u64>().map(|n| (n >> 22) % 6).unwrap_or(0);
            Some(format!("https://cdn.discordapp.com/embed/avatars/{index}.png"))
        }
    }
}

async fn connect(client_id: &str) -> Result<(Box<dyn IpcStream>, String, Option<String>), ConnectError> {
    let mut stream = open_ipc().await.map_err(|_| ConnectError::NotRunning)?;
    write_frame(&mut stream, OP_HANDSHAKE, &json!({ "v": 1, "client_id": client_id }))
        .await
        .map_err(|e| ConnectError::Failed(e.to_string()))?;
    let (op, v) = match timeout(Duration::from_secs(5), read_frame(&mut stream)).await {
        Ok(Ok(f)) => f,
        Ok(Err(e)) => return Err(ConnectError::Failed(e.to_string())),
        Err(_) => return Err(ConnectError::Failed("handshake timed out".into())),
    };
    match op {
        OP_FRAME if v["evt"] == "READY" => {
            let user = &v["data"]["user"];
            let name = user["global_name"]
                .as_str()
                .or(user["username"].as_str())
                .unwrap_or("unknown user")
                .to_owned();
            Ok((stream, name, avatar_url(user)))
        }
        OP_CLOSE => Err(ConnectError::Rejected(
            v["message"].as_str().map(str::to_owned).unwrap_or_else(|| format!("code {}", v["code"])),
        )),
        _ => Err(ConnectError::Failed(format!("unexpected handshake reply (op {op})"))),
    }
}

enum SessionEnd {
    Shutdown,
    ClientIdChanged,
    Lost(String),
}

impl DiscordWorker {
    pub fn start(self, sink: impl Fn(DiscordEvent) + Send + Sync + 'static) {
        tauri::async_runtime::spawn(self.run(sink));
    }

    async fn run(mut self, sink: impl Fn(DiscordEvent) + Send + Sync + 'static) {
        let mut status = DiscordStatus::Disconnected;
        let mut last_problem: Option<String> = None;
        let set_status = |status: &mut DiscordStatus, new: DiscordStatus| {
            if *status != new {
                *status = new.clone();
                sink(DiscordEvent::Status(new));
            }
        };

        loop {
            if *self.shutdown_rx.borrow() {
                self.done.notify_one();
                return;
            }
            let client_id = self.client_id_rx.borrow_and_update().clone();

            match connect(&client_id).await {
                Ok((stream, user, avatar)) => {
                    last_problem = None;
                    sink(DiscordEvent::Log("info", format!("Connected to Discord as {user}")));
                    set_status(&mut status, DiscordStatus::Connected { user, avatar });
                    match self.session(stream, &sink).await {
                        SessionEnd::Shutdown => {
                            self.done.notify_one();
                            return;
                        }
                        SessionEnd::ClientIdChanged => {
                            sink(DiscordEvent::Log("info", "Application ID changed, reconnecting to Discord".into()));
                            set_status(&mut status, DiscordStatus::Disconnected);
                            continue;
                        }
                        SessionEnd::Lost(why) => {
                            sink(DiscordEvent::Log("warn", format!("Lost connection to Discord ({why}). Retrying every 5 seconds.")));
                            set_status(&mut status, DiscordStatus::Disconnected);
                            last_problem = Some("lost".into());
                        }
                    }
                }
                Err(ConnectError::NotRunning) => {
                    if last_problem.as_deref() != Some("not-running") && last_problem.as_deref() != Some("lost") {
                        sink(DiscordEvent::Log("warn", "Discord isn't running. Waiting for it to start.".into()));
                    }
                    last_problem = Some("not-running".into());
                    set_status(&mut status, DiscordStatus::Disconnected);
                }
                Err(ConnectError::Rejected(msg)) => {
                    let line = format!("Discord rejected Application ID {client_id}: {msg}");
                    if last_problem.as_deref() != Some(line.as_str()) {
                        sink(DiscordEvent::Log("error", line.clone()));
                    }
                    last_problem = Some(line);
                    set_status(&mut status, DiscordStatus::Rejected(msg));
                }
                Err(ConnectError::Failed(msg)) => {
                    let line = format!("Couldn't connect to Discord: {msg}");
                    if last_problem.as_deref() != Some(line.as_str()) {
                        sink(DiscordEvent::Log("warn", line.clone()));
                    }
                    last_problem = Some(line);
                    set_status(&mut status, DiscordStatus::Disconnected);
                }
            }

            tokio::select! {
                _ = sleep(RETRY_EVERY) => {}
                _ = self.client_id_rx.changed() => {}
                _ = self.shutdown_rx.changed() => {}
            }
        }
    }

    async fn session(&mut self, stream: Box<dyn IpcStream>, sink: &(impl Fn(DiscordEvent) + Send + Sync)) -> SessionEnd {
        let (mut rd, mut wr) = tokio::io::split(stream);
        let (frame_tx, mut frame_rx) = mpsc::channel(16);
        let reader = tauri::async_runtime::spawn(async move {
            loop {
                let r = read_frame(&mut rd).await;
                let failed = r.is_err();
                if frame_tx.send(r).await.is_err() || failed {
                    break;
                }
            }
        });

        let mut nonce: u64 = 0;
        let mut last_sent: Option<Instant> = None;
        // Always (re)send the current activity on a fresh connection.
        let mut pending = self.activity_rx.borrow().is_some();

        let end = loop {
            let ready_at = last_sent.map(|t| t + MIN_SEND_INTERVAL).unwrap_or_else(Instant::now);
            tokio::select! {
                _ = self.shutdown_rx.changed() => {
                    let _ = send_activity(&mut wr, None, &mut nonce).await;
                    // Give Discord a moment to process the clear before the pipe closes.
                    sleep(Duration::from_millis(200)).await;
                    break SessionEnd::Shutdown;
                }
                _ = self.client_id_rx.changed() => {
                    let _ = send_activity(&mut wr, None, &mut nonce).await;
                    let _ = write_frame(&mut wr, OP_CLOSE, &json!({})).await;
                    break SessionEnd::ClientIdChanged;
                }
                _ = self.activity_rx.changed() => { pending = true; }
                _ = sleep_until(ready_at), if pending => {
                    let activity = self.activity_rx.borrow_and_update().clone();
                    if let Err(e) = send_activity(&mut wr, activity.as_ref(), &mut nonce).await {
                        break SessionEnd::Lost(e.to_string());
                    }
                    last_sent = Some(Instant::now());
                    pending = false;
                }
                frame = frame_rx.recv() => match frame {
                    Some(Ok((OP_PING, v))) => {
                        if let Err(e) = write_frame(&mut wr, OP_PONG, &v).await {
                            break SessionEnd::Lost(e.to_string());
                        }
                    }
                    Some(Ok((OP_CLOSE, v))) => {
                        break SessionEnd::Lost(v["message"].as_str().unwrap_or("closed by Discord").to_owned());
                    }
                    Some(Ok((OP_FRAME, v))) => {
                        if v["evt"] == "ERROR" {
                            let msg = v["data"]["message"].as_str().unwrap_or("unknown error");
                            sink(DiscordEvent::Log("warn", format!("Discord rejected the activity: {msg}")));
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => break SessionEnd::Lost(e.to_string()),
                    None => break SessionEnd::Lost("pipe closed".into()),
                }
            }
        };
        reader.abort();
        end
    }
}

async fn send_activity<W: AsyncWrite + Unpin>(w: &mut W, activity: Option<&Activity>, nonce: &mut u64) -> std::io::Result<()> {
    *nonce += 1;
    let payload = json!({
        "cmd": "SET_ACTIVITY",
        "args": {
            "pid": std::process::id(),
            "activity": activity.map(Activity::to_discord_json),
        },
        "nonce": format!("up-{nonce}"),
    });
    write_frame(w, OP_FRAME, &payload).await
}
