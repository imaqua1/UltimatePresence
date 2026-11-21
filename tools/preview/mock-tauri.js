// Fake Tauri bridge for previewing the window in a normal browser (dev only).
// Scenario via ?state=live|testing|vfx|paused|waiting|idle|error|nodiscord|rejected|held
// The template renderer below is a simplified copy of src-tauri/src/{template,studio}.rs
// so previews behave; the real app always uses the Rust version.
(function () {
  const params = new URLSearchParams(location.search);
  const scenario = params.get("state") || "live";
  const APP_ID = "1028311936854675458";
  const now = Date.now();

  const raws = {
    editing: { details: "DataStoreHandler (Line 42 / 310)", state: "Workspace: Obby Tycoon", largeImage: "serverscript", largeText: "Editing a SERVER Script" },
    developing: { details: "Developing", state: "Workspace: Obby Tycoon", largeImage: "studio", largeText: "UltimatePresence by aqua" },
    testing: { details: "Testing", state: "Workspace: Obby Tycoon", largeImage: "studio", largeText: "UltimatePresence by aqua", smallImage: "playtesticon", smallText: "Testing" },
    animating: { details: "Animating", state: "Workspace: Obby Tycoon", largeImage: "studio", largeText: "Moon Animator 2", smallImage: "moonanimator", smallText: "Animating" },
    vfx: { details: "Making VFX", state: "Workspace: Obby Tycoon", largeImage: "studio", largeText: "Qwinkle's Part-Icles", smallImage: "qwinkleparticles", smallText: "Making VFX" },
    combo: { details: "Animating & making VFX", state: "Workspace: Obby Tycoon", largeImage: "studio", largeText: "Eclipse Animator + Qwinkle's Part-Icles", smallImage: "animatingvfx", smallText: "Animating & making VFX" },
  };
  for (const r of Object.values(raws)) Object.assign(r, { startTimestamp: now - (23 * 60 + 14) * 1000, buttons: [] });

  const PRESETS = {
    classic: { editing: ["{script}[ (Line {line} / {lines})]", "[Workspace: {place}]"], developing: ["Developing", "[Workspace: {place}]"], testing: ["Testing", "[Workspace: {place}]"], animating: ["Animating", "[Workspace: {place}]"], vfx: ["Making VFX", "[Workspace: {place}]"], combo: ["Animating & making VFX", "[Workspace: {place}]"] },
    minimal: { editing: ["Scripting", "{place}"], developing: ["Building", "{place}"], testing: ["Playtesting", "{place}"], animating: ["Animating", "{place}"], vfx: ["Making VFX", "{place}"], combo: ["Animating & VFX", "{place}"] },
    detailed: { editing: ["Editing {script}", "[Line {line} of {lines} · ]{place}"], developing: ["Building in Studio", "{place}"], testing: ["Playtesting", "{place}"], animating: ["Animating[ with {tool}]", "{place}"], vfx: ["Making VFX[ with {tool}]", "{place}"], combo: ["Animating & making VFX", "[{tool} · ]{place}"] },
  };

  function renderTemplate(t, v) {
    let out = "", group = null;
    for (let i = 0; i < t.length; i++) {
      const c = t[i];
      if (c === "[" && !group) { group = { text: "", ok: true }; continue; }
      if (c === "]" && group) { if (group.ok) out += group.text; group = null; continue; }
      if (c === "{") {
        const end = t.indexOf("}", i);
        if (end < 0) { (group ? (group.text += c) : (out += c)); continue; }
        const val = v(t.slice(i + 1, end).trim());
        if (group) { if (!val) group.ok = false; group.text += val || ""; } else out += val || "";
        i = end; continue;
      }
      if (group) group.text += c; else out += c;
    }
    out = out.replace(/\s+/g, " ").replace(/\(\s*\)/g, "").replace(/^[\s·•|,:–—-]+|[\s·•|,:–—-]+$/g, "").replace(/\s+/g, " ").trim();
    return out ? out[0].toUpperCase() + out.slice(1) : "";
  }

  function kindOf(a) {
    if (a.details === "Testing") return "testing";
    if (a.details === "Animating & making VFX") return "combo";
    if (a.details === "Making VFX") return "vfx";
    if (a.details === "Animating") return "animating";
    if (a.details === "Developing") return "developing";
    return "editing";
  }

  function render(raw, s) {
    const kind = kindOf(raw);
    const m = /^(.*) \(Line (\d+) \/ (\d+)\)$/.exec(raw.details || "");
    const place = (raw.state || "").replace(/^Workspace:\s*/, "");
    const types = { serverscript: "server script", clientscript: "local script", modulescript: "module script" };
    const v = (n) => ({
      action: { editing: "Editing", developing: "Building", testing: "Playtesting", animating: "Animating", vfx: "Making VFX", combo: "Animating & making VFX" }[kind],
      script: m ? (s.hideScript ? "a script" : m[1]) : null,
      line: m && !s.hideLines ? m[2] : null,
      lines: m && !s.hideLines ? m[3] : null,
      type: types[raw.largeImage] || null,
      place: place ? (s.hidePlace ? s.placeAlias || "a private project" : place) : null,
      tool: ["animating", "vfx", "combo"].includes(kind) ? raw.largeText : null,
      animator: kind === "animating" ? raw.largeText : null,
    })[n];
    const custom = s.style === "custom" && s.templates[kind] && (s.templates[kind].top || s.templates[kind].bottom) ? s.templates[kind] : null;
    const [top, bottom] = custom ? [custom.top, custom.bottom] : PRESETS[s.style === "custom" ? "classic" : s.style][kind];
    const img = (s.images || {})[kind] || {};
    return {
      details: renderTemplate(top, v) || null,
      state: renderTemplate(bottom, v) || null,
      startTimestamp: s.timer === "off" ? null : raw.startTimestamp,
      largeImage: img.large || (s.scriptIcons ? raw.largeImage : "studio"),
      largeText: img.largeText ? renderTemplate(img.largeText, v) : /^StudioPresence by/.test(raw.largeText || "") ? "UltimatePresence by aqua" : raw.largeText,
      smallImage: s.badges ? img.small || raw.smallImage || null : null,
      smallText: s.badges ? (img.smallText ? renderTemplate(img.smallText, v) : raw.smallText || null) : null,
      buttons: s.buttons.slice(),
    };
  }

  const defaults = () => ({
    enabled: true, applicationId: "", style: "classic", templates: {}, scriptIcons: true, badges: true, stateIcons: true, images: {}, timer: "session",
    hideScript: false, hidePlace: false, hideLines: false, hideTools: false, placeAlias: "", hiddenPlaces: [],
    showTesting: true, idleMinutes: 0, buttons: [],
  });
  const source = (id, name, available) => ({
    id, name, available,
    defaultApplicationId: available ? APP_ID : "",
    settings: Object.assign(defaults(), { applicationId: available ? APP_ID : "" }),
    status: available ? "idle" : "soon", statusText: available ? "Not running" : "Coming soon", detail: null, activity: null,
  });

  const s = {
    version: "0.3.0", discordStatus: "connected", discordUser: "aqua", discordAvatar: "https://cdn.discordapp.com/embed/avatars/3.png", discordMessage: null, updateAvailable: params.get("update") || null, discordApplicationId: APP_ID,
    paused: false, autostart: true, openWindowOnLogin: false, activeSource: null, shown: null,
    sources: [source("roblox-studio", "Roblox Studio", true), source("blender", "Blender", true), source("substance-painter", "Substance Painter", false), source("figma", "Figma", false)],
    focused: null,
    history: [], startedMs: now - 52 * 60 * 1000,
  };
  const studio = s.sources[0];
  const blender = s.sources[1];
  blender.settings.applicationId = "";
  blender.defaultApplicationId = "";
  const BL = { modeling: "Modeling", sculpting: "Sculpting", uv: "UV unwrapping", texturing: "Texture painting", shading: "Shading", animating: "Animating", rigging: "Rigging", rendering: "Rendering", compositing: "Compositing", geonodes: "Building geometry nodes", drawing: "Drawing", scripting: "Scripting", video: "Editing video", idle: "Idle" };
  const blenderActivity = (kind) => ({
    details: kind === "idle" ? "Idle" : kind === "rendering" ? "Rendering spaceship_v3" : BL[kind] + " Hull",
    state: kind === "rendering" ? "Frame 120 of 250" : "spaceship_v3",
    startTimestamp: now - 48 * 60 * 1000,
    largeImage: kind === "idle" ? "icon_idle" : "bl_" + kind, largeText: "UltimatePresence by aqua",
    smallImage: "bl_blender", smallText: "Blender", buttons: [],
  });
  let blenderLive = false;
  let raw = null;
  let forced = null;
  switch (scenario) {
    case "live": raw = raws.editing; break;
    case "testing": raw = raws.testing; break;
    case "vfx": raw = raws.vfx; break;
    case "combo": raw = raws.combo; break;
    case "paused": raw = raws.editing; s.paused = true; break;
    case "waiting": forced = "waiting"; break;
    case "error": forced = "error"; break;
    case "held": raw = raws.editing; studio.settings.hiddenPlaces = ["Obby Tycoon"]; break;
    case "nodiscord": raw = raws.editing; s.discordStatus = "disconnected"; s.discordUser = null; break;
    case "blender": blenderLive = true; blender.settings.applicationId = "1572631495351205898"; s.focused = "blender"; break;
    case "rejected": s.discordStatus = "rejected"; s.discordUser = null; s.discordMessage = "Invalid Client ID"; break;
  }
  const mins = (m) => now - m * 60 * 1000;
  s.history = raw ? [
    { timeMs: mins(2), source: "roblox-studio", title: "DataStoreHandler", subtitle: "Workspace: Obby Tycoon" },
    { timeMs: mins(9), source: "roblox-studio", title: "Testing", subtitle: "Workspace: Obby Tycoon" },
    { timeMs: mins(17), source: "roblox-studio", title: "Animating", subtitle: "Workspace: Obby Tycoon" },
    { timeMs: mins(31), source: "roblox-studio", title: "PlayerController", subtitle: "Workspace: Obby Tycoon" },
    { timeMs: mins(44), source: null, title: "Stopped sharing", subtitle: null },
    { timeMs: mins(50), source: "roblox-studio", title: "Developing", subtitle: "Workspace: Obby Tycoon" },
  ] : [];

  function recompute() {
    const st = studio.settings;
    const hidden = raw && st.hiddenPlaces.some((p) => p.toLowerCase() === "obby tycoon");
    studio.activity = raw && st.enabled && !hidden ? render(raw, st) : null;
    studio.detail = null;
    if (forced === "error") { studio.status = "error"; studio.statusText = "Port 4455 is busy"; studio.detail = "Port 4455 is already in use. Close the old StudioPresence server (studiopresence-win.exe) or whatever else is using it. Retrying…"; }
    else if (forced === "waiting") { studio.status = "waiting"; studio.statusText = "Open, waiting for the plugin"; }
    else if (!st.enabled) { studio.status = "disabled"; studio.statusText = "Turned off"; }
    else if (hidden) { studio.status = "held"; studio.statusText = studio.detail = "Hidden while you’re in this place"; }
    else if (raw) { studio.status = "sharing"; studio.statusText = "Connected"; }
    else { studio.status = "idle"; studio.statusText = "Not running"; }
    blender.activity = blenderLive ? blenderActivity("sculpting") : null;
    blender.status = blenderLive ? "sharing" : "idle";
    blender.statusText = blenderLive ? "Connected" : "Not running";
    if (blenderLive && !s.paused) { s.shown = blender.activity; s.activeSource = "blender"; return; }
    s.shown = !s.paused && studio.activity ? studio.activity : null;
    s.activeSource = s.shown ? "roblox-studio" : null;
  }
  recompute();

  const t0 = now - 52 * 60 * 1000;
  const log = [
    ["info", "UltimatePresence 0.3.0 started"], ["info", "Listening for Roblox Studio on localhost:4455"],
    ["warn", "Discord isn't running. Waiting for it to start."], ["info", "Connected to Discord as aqua"],
    ["info", "Roblox Studio is running"], ["info", "Roblox Studio connected"], ["info", "Now showing Roblox Studio on Discord"],
    ["info", "Playtest started"], ["info", "Playtest ended"],
  ].map(([level, message], i) => ({ id: i + 1, timeMs: t0 + i * 190000, level, message }));

  const listeners = {};
  const emit = (ev, p) => (listeners[ev] || []).forEach((f) => f({ payload: p }));
  const clone = (o) => JSON.parse(JSON.stringify(o));

  window.__TAURI__ = {
    core: {
      async invoke(cmd, a = {}) {
        switch (cmd) {
          case "get_state": return clone(s);
          case "get_integrations": return { studioPlugin: params.get("plugin") || "installed", blenderAddon: params.get("blender") || "missing", blenderVersions: params.get("blender") === "installed" ? ["5.3"] : [] };
          case "install_integration": return a.which === "blender" ? "Installed for Blender 5.3. Restart Blender and it connects by itself." : "Plugin installed. Restart Roblox Studio to load it.";
          case "check_for_update": return s.updateAvailable;
          case "install_update": throw "Preview only: nothing to install";
          case "get_log": return clone(log);
          case "get_presets": {
            if (a.source === "blender") {
              const st = {}; for (const k of Object.keys(BL)) st[k] = { top: k === "idle" ? "Idle" : "{action}[ {object}]", bottom: "[{file}]" };
              return { styles: { classic: st, minimal: st, detailed: st }, kinds: Object.keys(BL) };
            }
            const styles = {};
            for (const [k, v] of Object.entries(PRESETS)) styles[k] = Object.fromEntries(Object.entries(v).map(([kind, [top, bottom]]) => [kind, { top, bottom }]));
            return { styles, kinds: Object.keys(raws) };
          }
          case "render_preview": {
            if (a.source === "blender") return blenderActivity(a.kind || "sculpting");
            const st = Object.assign(clone(studio.settings), a.patch || {});
            return render(a.kind ? raws[a.kind] : raw || raws.editing, st);
          }
          case "set_paused": s.paused = a.paused; break;
          case "set_autostart": s.autostart = a.enabled; break;
          case "update_settings": Object.assign(s, a.patch); break;
          case "update_source_settings": {
            const p = a.patch;
            if (p.applicationId && !/^\d{17,20}$/.test(p.applicationId)) throw "An Application ID is 17–20 digits.";
            if (p.buttons && p.buttons.some((b) => !/^https?:\/\//.test(b.url))) throw "Links need to start with https://";
            Object.assign((a.source === "blender" ? blender : studio).settings, clone(p));
            break;
          }
          case "reset_source_settings": studio.settings = Object.assign(defaults(), { applicationId: studio.settings.applicationId }); break;
          case "clear_log": log.length = 0; return;
          case "open_external": console.log("open", a.target); return;
          case "ui_ready": return;
          default: console.warn("mock: unhandled", cmd); return;
        }
        recompute();
        setTimeout(() => emit("state", clone(s)), 0);
        return cmd === "set_autostart" ? a.enabled : undefined;
      },
    },
    event: { async listen(ev, f) { (listeners[ev] = listeners[ev] || []).push(f); return () => {}; } },
  };
})();
