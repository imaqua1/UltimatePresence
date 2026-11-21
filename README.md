# UltimatePresence

Shows what you're working on in **Roblox Studio** and **Blender** on your Discord
profile. It sits in the system tray and updates itself.

- **Follows the app in front.** With Studio and Blender both open, Discord shows the one
  you're using. Click away to Discord or a browser and it keeps the last one.
- **Knows what you're doing.** In Studio: editing a script, building, playtesting,
  animating (Moon Animator 2, Eclipse) and making VFX (Qwinkle's Part-Icles). In Blender:
  modeling, sculpting, UVs, painting, shading, animating, rigging, rendering with
  progress, nodes and more.
- **Idle after 5 minutes** of not using the app (working in other programs doesn't count),
  with a Zzz icon.
- **Private when you want.** Hide script, place, file or object names, or never share
  certain places or .blend files.
- **Light.** Native Rust app: about 10 MB on disk, about 5 MB of memory in the tray.

> Substance Painter and Figma are on the way.

## Install

1. Download **`UltimatePresence_x.y.z_x64-setup.exe`** from the
   [latest release](https://github.com/imaqua1/UltimatePresence/releases/latest) and run it.
   It installs for your Windows user, so there's no admin prompt. Windows may warn that the
   installer isn't signed: click **More info → Run anyway**.
2. Open UltimatePresence and follow **Get set up** on its Home page:
   - **Discord**: keep the Discord desktop app open.
   - **Roblox Studio**: click *Install plugin*, then restart Studio.
   - **Blender**: click *Install script*, then restart Blender. It loads by itself every
     time Blender starts; there's nothing to enable.
3. That's it. Work as usual.

New versions install with one click from **Settings → Updates**; the app checks on its own.

## Using it

- **Tray icon**: orange while sharing, grey otherwise. Left-click opens the window;
  right-click to pause sharing, toggle *Start with Windows*, or quit.
- **Home**: your Discord card exactly as friends see it, and what each app is doing.
- **One page per app**: wording styles (Classic, Minimal, Detailed or your own), privacy,
  playtest and button options, with a live preview.
- Closing the window keeps the app in the tray. Quitting clears your status.

### Your own wording

Pick **Custom** on an app's Display tab and write the two lines Discord shows. Click a
`{value}` to insert it. Wrap part of a line in `[ ]` to drop it when a value inside is
hidden or empty: `{script}[ (Line {line} / {lines})]` shows just the script name when line
numbers are hidden.

## Privacy

Everything runs on your PC. The app listens only on `127.0.0.1:4455`, accepts nothing from
web pages, and talks to Discord through the desktop app. The only internet request it makes
is the update check to this repository's Releases.

## Troubleshooting

| You see | Do this |
| --- | --- |
| “Discord isn't open” | Start the Discord desktop app (the browser version can't show activities). |
| Studio shows nothing | Restart Studio after installing the plugin, and check *Plugins → Manage Plugins*. |
| Blender shows “waiting for the script” | Restart Blender after installing the script. |
| “Port 4455 is in use” | Another presence server is running (for example StudioPresence's). Close it. |

The **Activity log** in the window lists connections and problems.

## Credits

Made by **aqua**. The Roblox Studio plugin builds on code from
[StudioPresence](https://github.com/iArxic/StudioPresence) by iArxic (ISC). Fonts:
Instrument Serif and Figtree (SIL Open Font Licence). See
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

Licensed under the [ISC licence](LICENSE). Building from source and releasing:
[docs/development.md](docs/development.md).
