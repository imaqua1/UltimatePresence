//! OS-specific helpers.

/// Executable names of all running processes.
#[cfg(windows)]
pub fn process_names() -> Vec<String> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };
    let mut out = Vec::new();
    // SAFETY: standard Toolhelp snapshot iteration; the handle is closed below.
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return out;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snap, &mut entry) != 0 {
            loop {
                let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                out.push(String::from_utf16_lossy(&entry.szExeFile[..len]));
                if Process32NextW(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
    }
    out
}

#[cfg(unix)]
pub fn process_names() -> Vec<String> {
    std::process::Command::new("ps")
        .args(["-A", "-c", "-o", "comm="])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().map(|l| l.trim().to_owned()).collect())
        .unwrap_or_default()
}

/// Executable name of the app whose window is in front (e.g. "blender.exe").
#[cfg(windows)]
pub fn foreground_process_name() -> Option<String> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    // SAFETY: plain Win32 queries; the process handle is closed before returning.
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return None;
        }
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut len);
        CloseHandle(handle);
        if ok == 0 {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        path.rsplit(['\\', '/']).next().map(str::to_owned)
    }
}

#[cfg(not(windows))]
pub fn foreground_process_name() -> Option<String> {
    None
}

/// Time since the last keyboard or mouse input anywhere on this PC.
#[cfg(windows)]
pub fn input_idle_for() -> std::time::Duration {
    use windows_sys::Win32::System::SystemInformation::GetTickCount;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    let mut info = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    // SAFETY: plain Win32 queries into a correctly sized struct.
    let ticks = unsafe {
        if GetLastInputInfo(&mut info) == 0 {
            return std::time::Duration::ZERO;
        }
        GetTickCount()
    };
    // Both are milliseconds since boot and wrap after 49.7 days, hence wrapping_sub.
    std::time::Duration::from_millis(ticks.wrapping_sub(info.dwTime) as u64)
}

#[cfg(not(windows))]
pub fn input_idle_for() -> std::time::Duration {
    std::time::Duration::ZERO
}

pub fn is_studio_process(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n == "robloxstudiobeta.exe" || n == "robloxstudio.exe" || n == "robloxstudio"
}

pub fn is_blender_process(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n == "blender.exe" || n == "blender"
}

/// Which source an executable belongs to.
pub fn source_for_process(name: &str) -> Option<&'static str> {
    if is_studio_process(name) {
        Some(crate::sources::ROBLOX_STUDIO)
    } else if is_blender_process(name) {
        Some(crate::sources::BLENDER)
    } else {
        None
    }
}
