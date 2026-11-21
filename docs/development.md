# Development

## Project layout

```
src-tauri/                 the app (Rust, Tauri v2)
  src/main.rs              setup, plugins, window
  src/sources.rs           catalog of apps and their Discord applications; start here to add one
  src/state.rs             central state; picks the app in front and what Discord shows
  src/studio.rs            Roblox Studio: HTTP server on 127.0.0.1/::1:4455, plugin protocol, wording
  src/blender.rs           Blender: payloads from the script, 13 categories, wording
  src/template.rs          {value} / [optional] text templates shared by every app
  src/discord.rs           Discord IPC client, reconnects on its own
  src/activity.rs          activity → SET_ACTIVITY payload (length limits etc.)
  src/integrations.rs      one-click install of the Studio plugin and Blender script
  src/updates.rs           update checks against GitHub Releases
  src/platform.rs          process list and foreground window (Windows)
  src/settings.rs          settings.json in %APPDATA%\com.ultimatepresence.desktop
  src/tray.rs, commands.rs tray menu, actions shared by tray and window
  icons/                   generated app and tray icons
ui/                        the window: plain HTML/CSS/JS, no build step; per-app copy lives in APPS in app.js
integrations/
  roblox-studio/           the Studio plugin source (built into dist/UltimatePresence.rbxmx)
  blender/                 the Blender startup script
design/                    logo and tray SVGs, Instrument Serif source fonts
scripts/                   build, plugin, logo, icon and Discord image generators
dev/                       UI preview harness and a plugin simulator
docs/design.md             design principles for the window
```

## Building

Prerequisites (Windows): [Rust](https://rustup.rs) (stable, MSVC), Visual Studio 2022 Build
Tools with *Desktop development with C++*, [Node.js](https://nodejs.org) 18+.

| Command | What it does |
| --- | --- |
| `npm install` | Installs the Tauri CLI |
| `npm run plugin` | Builds `dist/UltimatePresence.rbxmx` (run once before `dev`; no Rojo needed) |
| `npm run dev` | Debug build with a console window |
| `npm run release` | Tests, signed release build; installer and portable exe land in `dist/` |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Unit tests (protocol quirks, wording, Blender categories) |

`npm run plugin:install` also copies the plugin into your Studio Plugins folder.

## Working on the window

`npm run preview` serves the real `ui/` files with a fake Tauri bridge at
`http://localhost:5178/dev/preview/?state=live`. Other states: `testing`, `vfx`, `held`,
`paused`, `waiting`, `idle`, `error`, `nodiscord`, `rejected`. It runs alongside an installed
copy. Design rules are in [design.md](design.md).

With the app running, `dev/simulate-plugin.ps1` sends the plugin's requests (edit, playtest,
CLOSE, bad bodies, a browser request) and checks the result via `GET /status`.

## Logo and icons

- `npm run logo` draws `design/icon.svg` and the tray SVGs from Instrument Serif's real “U”.
- `npm run icons` turns them into every app and tray icon in `src-tauri/icons`.
- `npm run discord-images` draws the Rich Presence pictures into `dist/discord-assets/`
  for uploading to the Discord applications (below). They are not bundled with the app.

## Discord applications

The applications are built in (`src-tauri/src/sources.rs`); users set nothing up. Each
application's name is what Discord shows after “Playing”, and it holds the pictures
(Developer Portal → Rich Presence → Art Assets). Re-upload a PNG under the same name to
change a picture; nobody has to update the app.

| App | Application ID | Pictures |
| --- | --- | --- |
| Roblox Studio | `1572631495351205898` | `studio` (badge), `serverscript`, `clientscript`, `modulescript`, `icon_building`, `icon_playtest`, `icon_animating`, `icon_vfx`, `icon_combo`, `icon_idle` |
| Blender | `1573670951235620886` | `bl_blender` (badge), `bl_modeling`, `bl_sculpting`, `bl_uv`, `bl_painting`, `bl_shading`, `bl_animating`, `bl_rendering`, `bl_nodes`, `icon_idle` |

## Local HTTP API

| Request | Response |
| --- | --- |
| `POST /` | Roblox Studio plugin activity (StudioPresence-compatible protocol) |
| `POST /blender` | Blender script context |
| `GET /status` | JSON: version, Discord state, active app, what's shown |

Loopback only. Requests with an `Origin` header are refused, so web pages can't reach it.

## Releasing

1. Bump `version` in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and `package.json`.
2. Commit, then `git tag v0.6.1` and `git push origin main v0.6.1`.
3. The *Release* workflow tests, builds, signs and publishes. Installed apps pick it up
   within a few hours, or straight away via *Check now*.

Updates are signed. The private key is at `%USERPROFILE%\.tauri\ultimatepresence.key` and
in the `TAURI_SIGNING_PRIVATE_KEY` repository secret; the public key is in
`tauri.conf.json`. **Keep a backup of the private key**: without it, existing installs can
never update again.

## Adding an app

Substance Painter and Figma are already in the catalog as planned, with sidebar entries and
pages. To make one live: write its source next to `blender.rs` (a local endpoint or a
process/window watcher), set `available: true` and its Discord application in
`sources.rs`, wire it into `state.rs`, and give it real controls in `ui/app.js`.
