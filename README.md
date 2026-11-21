# UltimatePresence

A small Windows tray app that shows what you're doing in **Roblox Studio** and
**Blender** on your Discord profile. It replaces the console-window server from
[StudioPresence](https://github.com/iArxic/StudioPresence) by iArxic and also works with
the **original StudioPresence plugin, unchanged**.

- **The app you have in front wins.** With Studio and Blender both open, Discord shows
  the one whose window is focused; switch to Discord or a browser and it keeps the last one.
- **Idle after 5 minutes** without changes: your status says “Idle” with a Zzz icon.
- **Fixed icons**: each state has its own picture, the small badge is the app's own icon,
  and hovering says “UltimatePresence by aqua” (large) or the app's name (badge).

- Lives in the system tray. No console window. Starting it again just opens the window.
- Reconnects to Discord on its own, and is fine if Discord isn't open.
- Shows your Discord profile card exactly as friends see it, with one page per app.
- Per-app settings: wording styles, privacy (hide file, place, script or object names),
  private files/places, profile buttons.
- **Updates itself** from this repo's Releases.
- Native Rust app (Tauri v2): about 10 MB on disk, about 5 MB of memory in the tray.

> Roblox Studio and Blender work today. Substance Painter and Figma are on the way.

## Install

1. Download **`UltimatePresence_x.y.z_x64-setup.exe`** from the
   [latest release](https://github.com/imaqua1/UltimatePresence/releases/latest) and run it.
   It installs for your Windows user, so there's no admin prompt. The installer isn't
   code-signed yet, so Windows may warn: **More info → Run anyway**.
2. Open UltimatePresence. Its Home page has **Get set up**:
   - **Discord**: keep the Discord desktop app open.
   - **Roblox Studio plugin**: click *Install plugin*, then restart Roblox Studio.
   - **Blender script**: click *Install script*, then restart Blender. It runs by itself
     every time Blender starts; there's nothing to enable.
3. Done. Work as usual and your Discord status follows whichever app you have in front.

Updates arrive by themselves: the app checks GitHub and offers **Install and restart**
(Settings → Updates).

## Blender

The Blender script (`blender/ultimate_presence.py`) goes in Blender's startup scripts
folder, so it loads automatically. The app's *Install script* button does that for every
Blender version you have. You can also install it the classic way: *Edit → Preferences →
Add-ons → Install from Disk*. It only talks to `127.0.0.1:4455` on your own PC.

What it detects, from Blender's mode, workspace and editors: modeling, sculpting, UV
unwrapping, texture painting, shading, animating, rigging, rendering (with frame
progress), compositing, geometry nodes, Grease Pencil drawing, scripting and video
editing. Wording values: `{action}` `{object}` `{file}` `{mode}` `{workspace}`
`{frame}` `{frames}` `{progress}` `{version}`. Privacy can hide object names, the file
name, frame numbers, and specific .blend files entirely.

The script sends raw context and the app decides the category, so detection can improve
without reinstalling the script. Tested against Blender 5.3's default workspaces.

## Releasing a new version (for maintainers)

1. Bump `version` in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and `package.json`.
2. Commit, then tag and push: `git tag v0.6.1` and `git push origin main v0.6.1`.
3. The *Release* workflow tests, builds and signs everything and publishes the release.
   Installed apps see it within a few hours (or right away via *Check now*).

Update packages are signed. The private key lives in `%USERPROFILE%\.tauri\` on the
maintainer's PC and as the `TAURI_SIGNING_PRIVATE_KEY` secret in this repo; the public key
is in `tauri.conf.json`. Lose the private key and existing installs can't update anymore,
so keep a backup of it somewhere safe.

## Roblox Studio

### The UltimatePresence plugin

A fork of StudioPresence's plugin (source in `plugin/`, ISC, credits kept). It sends the
same messages, so the original server still understands it. What it adds:

| Tool | How it's detected | Shows as |
| --- | --- | --- |
| Moon Animator 2 | `MoonAnimator2MouseFilter` in the camera (as before) | Animating |
| Eclipse Animator | `CoreGui["Eclipse Editor [Animator] …"]` is enabled while the editor is open | Animating |
| Qwinkle's Part-Icles | `CoreGui.PartIclesUI` exists while it's open | Making VFX |
| An animator **and** Part-Icles | both of the above | Animating & making VFX |

It also fixes the original's small-badge tooltip field, and resends when only the tool
changes (the original skipped updates whose top line hadn't changed).

Build it with `npm run plugin` (writes `dist/UltimatePresence.rbxmx`, no Rojo needed).
`npm run plugin:install` also copies it into your Plugins folder and renames the
original `studiopresence.rbxm` to `.disabled`. To go back, delete
`UltimatePresence.rbxmx` and remove `.disabled` from the original's name.

### Coming from the original StudioPresence server?

1. Close `studiopresence-win.exe` (its console window, or Task Manager).
2. Delete its `startup.vbs` from your Startup folder (press `Win + R`, type `shell:startup`).
3. Turn on **Start with Windows** in UltimatePresence instead.

UltimatePresence warns you in its log if the old server is still running. Both use
port 4455, so you only want one of them.

## Using it

**Tray icon** (orange while sharing, grey otherwise). Right-click it for:

| Item | What it does |
| --- | --- |
| Discord / Roblox Studio / Showing… | Live status (read-only) |
| Pause sharing | Clears your status without closing Studio |
| Start with Windows | Adds or removes a `HKCU\…\Run` entry that starts the app minimized to the tray |
| Open UltimatePresence | Opens the window (a left-click on the icon does the same) |
| Quit | Clears your status and exits |

**The window**:

- **Home**: a headline that tells you where you stand (“You're *live* on Discord.”),
  your Discord profile card exactly as friends see it, a timeline of this session,
  and your apps, each with its own on/off switch. If something is wrong (Discord
  closed, plugin missing, port busy, held back by your settings) it says what and why.
- **One page per app** with tabs. Roblox Studio has:
  - **Overview**: what it's sending right now, connection status, and which tools it
    detects (scripts, playtests, Moon Animator; Eclipse Animator and Qwinkle Particles
    are in progress).
  - **Display**: four styles (Classic, Minimal, Detailed, Custom), your own wording
    for each state with clickable `{tokens}`, script-type images, status badges, and
    what the timer counts.
  - **Privacy**: hide script names, the place name (with your own replacement text),
    or line numbers; plus places that are never shared at all.
  - **Behaviour**: show or skip playtests, hide after inactivity (5 min to 1 hour),
    up to two profile buttons, and a reset.
  - A **live preview** stays pinned beside Display, Privacy and Behaviour, rendered by
    the same code that talks to Discord. Preview any state (editing, playtesting,
    animating, VFX) even when Studio is closed.
- **Blender and Figma** pages show what they'll do and their planned settings.
- **Settings**: start with Windows, open the window at sign-in, theme (Dark / Light /
  System), which app wins when several are active, and your settings folder.
- **Activity log**: connections, plugin events and problems, with a *Problems* filter.
  The sidebar shows a count when new warnings arrive.
- The **you card** at the bottom of the sidebar shows your status and pauses or resumes
  sharing from anywhere.

### Custom wording

Pick **Custom** on the Display tab and write the two lines Discord shows for each
state. Values: `{action}` `{script}` `{line}` `{lines}` `{type}` `{place}` `{tool}`.
Wrap a part in `[ ]` to drop it when a value inside is hidden or missing, for example
`{script}[ (Line {line} / {lines})]` shows just the script name when line numbers are
hidden.

### Icons

Icons are fixed, so there's nothing to pick. Each state has its own large picture
(the script type while editing a script, a Zzz when idle), and the small badge is always
the app's own icon. Hovering the picture says “UltimatePresence by aqua”; hovering the
badge says the app's name.

Closing the window doesn't quit the app; it keeps running in the tray. The window is
destroyed when closed, so it uses no memory in the background.

**When does the status clear?**

- The plugin sends `CLOSE` (Studio closing, plugin disabled) → cleared immediately.
- Studio crashes or is killed → cleared within about 10 seconds. The app checks that
  `RobloxStudioBeta.exe` is gone *and* no update has arrived for 10 s.
  (The plugin only sends updates when something changes, so silence alone doesn't
  mean you've left. Sitting on one line of code keeps your status up.)
- You pause sharing or quit the app.
- Blender: the add-on says goodbye when Blender closes; if it crashes, the status
  clears within a few seconds of `blender.exe` disappearing (or 30 s without a heartbeat).

## Discord applications (for maintainers)

Users don't set anything up: the Discord applications are built in
(`src-tauri/src/sources.rs`). Their names are what Discord shows after “Playing”, and
they hold the images (Developer Portal → Rich Presence → Art Assets):

| App | Application ID | Images |
| --- | --- | --- |
| Roblox Studio | `1572631495351205898` | `studio` (badge), `serverscript`, `clientscript`, `modulescript`, `icon_building`, `icon_playtest`, `icon_animating`, `icon_vfx`, `icon_combo`, `icon_idle` |
| Blender | `1573670951235620886` | `bl_blender` (badge), `bl_modeling`, `bl_sculpting`, `bl_uv`, `bl_painting`, `bl_shading`, `bl_animating`, `bl_rendering`, `bl_nodes`, `icon_idle` |

The icons are drawn by `tools/make-state-icons.mjs` and `tools/make-blender-icons.mjs`
(512×512 PNGs in `dist/discord-assets/`). Re-upload a PNG under the same name to change
an icon; nobody needs to update the app for that.

## Build from source

Prerequisites (Windows):

- [Rust](https://rustup.rs) (stable, MSVC toolchain)
- Visual Studio 2022 Build Tools with the **Desktop development with C++** workload
- [Node.js](https://nodejs.org) 18 or newer (only for the Tauri CLI)
- WebView2 runtime (already part of Windows 10/11)

```bash
npm install
```

```bash
npm run dev
```

`npm run dev` runs a debug build with a console window for logs.

```powershell
powershell -ExecutionPolicy Bypass -File build.ps1
```

`build.ps1` runs the tests and a release build, and puts the installer and the
portable exe in `dist/`. `npm run build` does the Tauri build alone; output lands in
`src-tauri/target/release/` and `src-tauri/target/release/bundle/nsis/`.

### Tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Covers the plugin protocol quirks, timeout rule, privacy mode and Discord payload shape.

```powershell
powershell -ExecutionPolicy Bypass -File tools\simulate-plugin.ps1
```

With the app running, this sends the same requests the plugin sends (edit, cursor
move, playtest hold, CLOSE, bad bodies, browser request) and checks the result through
`GET /status`. Add `-Keep` to leave a demo activity on your profile.

### Working on the window

The window is plain HTML/CSS/JS in `ui/`. To iterate on it without running the app
(or while your installed copy is running), use the preview: it loads the real `ui/`
files with a fake Tauri bridge and switchable states.

```bash
node tools/preview/server.mjs
```

Then open `http://localhost:5178/tools/preview/?state=live`. Other states: `testing`,
`vfx`, `held`, `paused`, `waiting`, `idle`, `error`, `nodiscord`, `rejected`. Settings
work against the fake state. Design context is in [PRODUCT.md](PRODUCT.md).

### Logo and icons

The logo is an orange italic serif “U” (Instrument Serif) with a presence dot on a black
tile. `node tools/make-logo.mjs <variant>` turns the font's real glyph outlines into
`assets/icon.svg` and the tray SVGs (variants: `italic-dot`, `roman`, `italic-light`,
`inverse`; a preview sheet is at `/tools/preview/logos.html`). Then `npm run icons`
regenerates every app and tray icon.

### Layout

```
src-tauri/src/
  main.rs       app setup, window, quit
  sources.rs    catalog of apps (live + planned); start here to add an app
  state.rs      central state; picks the active app, its Discord application and activity
  studio.rs     Roblox Studio source: HTTP server on 127.0.0.1/::1:4455, plugin quirks, timeout,
                parsing the plugin's text, styles/presets, privacy, idle and hidden places
  template.rs   the {token} / [optional] text templates every app uses
  discord.rs    Discord IPC client (named pipe / Unix socket), reconnect loop
  activity.rs   activity model → Discord SET_ACTIVITY payload (length limits etc.)
  tray.rs       tray icon + menu
  commands.rs   actions shared by tray and window
  platform.rs   process list (Studio detection)
  settings.rs   settings.json in %APPDATA%\com.ultimatepresence.desktop (per-app, migrates v0.1)
ui/             the window (plain HTML/CSS/JS, no build step); per-app copy lives in APPS in app.js
tools/          icon generator, plugin simulator, UI preview
assets/         icon sources
```

### Adding an app (Blender, Figma, …)

Blender and Figma are already in the catalog as planned sources, so they have
sidebar entries, pages and settings slots. To make one live:

1. Write its runtime next to `studio.rs`, for example a local endpoint for a Blender
   add-on, or a window-title watcher for Figma. Store its current activity and
   last-update time in `Core`.
2. In `sources.rs`, set `available: true` and give it a default Discord application
   with matching art assets.
3. In `state.rs`, return its activity from `source_activity`, its timestamp from
   `candidates`, and its status from `source_view`. Arbitration (most recent wins) and
   Discord application switching already work.
4. In `ui/app.js`, move its `planned` settings into real controls on a live page.

### Local HTTP API

| Request | Response |
| --- | --- |
| `POST /` with the plugin's JSON | `200 SET Activity` (same as the original server) |
| `GET /status` | JSON: app version, Discord state, paused, Studio enabled/privacy/status, active app, what's shown |

The server only listens on loopback and refuses requests that carry an `Origin`
header, so web pages can't change your Discord status.

## Roadmap

- Optional fork of the plugin that reads `GET /status` and shows the app's state inside Studio.
- Blender and other apps as additional sources.
- macOS build (the IPC client and autostart already support it; untested).

## Credits and licence

Based on [StudioPresence](https://github.com/iArxic/StudioPresence) by **iArxic**
(ISC licence). StudioPresence itself credits Rigid Studios for the original plugin.
UltimatePresence talks to the same plugin and uses the same default Discord application.

Licensed under the ISC licence. See [LICENSE](LICENSE) and
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
