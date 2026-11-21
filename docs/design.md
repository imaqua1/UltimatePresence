# UltimatePresence — product context

**Register:** product (app UI). Design serves the task.

## What it is
A Windows tray app that shows what a creator is doing in their tools (Roblox Studio today; Blender, Figma and others next) as Discord Rich Presence.

## Who uses it, and how
Hobbyist and indie creators (Roblox devs, 3D artists, designers), often teenagers to young adults, who live in Discord. They sit at a desk, usually evening, with a dark creative app on one monitor and Discord on the other. They open the window rarely and briefly: "am I showing anything, what does it say, who can see it?", then flip a switch and close it. The rest of the time the app is a tray icon.

## Jobs, in priority order
1. See at a glance what Discord is showing right now, and from which app.
2. Pause / resume sharing instantly.
3. Control privacy per app (hide file / place / script names).
4. Fix problems: Discord not running, plugin not connected, port busy.
5. Set it up once: start with Windows, custom Discord application per app.

## Personality
Quiet, precise, a little playful in the details. Feels like a native utility that belongs next to Discord and creative tools, not a web dashboard. Never shouts; status is legible from across the desk.

## Visual language (v0.3)
- Editorial dark, after the user's “Name Taste” artifact: **Instrument Serif** display
  type with one *italic orange* word per headline, **Figtree** for the interface.
- Warm near-black surfaces, orange as the single accent, green only for “live”.
- Roomy: 1180×780 window, big headlines, 20px+ radii, settings in grouped rows.
- Logo: orange italic serif “U” with a presence dot on a black tile.

## Constraints
- Tauri v2 WebView2 window, plain HTML/CSS/JS, no build step, no remote fonts (offline, strict CSP).
- Must stay tiny: window is created on demand and destroyed on close.
- Do not use other companies' logos; app icons are abstract glyphs with each app's own hue.
