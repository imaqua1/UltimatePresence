"use strict";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

/* ======================================================================
   App copy and metadata (presentation only; the app list itself comes from
   the Rust side, so new apps appear automatically).
   ==================================================================== */

const APPS = {
  "roblox-studio": {
    glyph: "cube",
    blurb: "Scripts, line numbers, playtests and the tools you build with.",
    kinds: {
      editing: "Editing a script",
      developing: "Building",
      testing: "Playtesting",
      animating: "Animating",
      vfx: "Making VFX",
      combo: "Animating + VFX",
      idle: "Idle",
    },
    tokens: ["action", "script", "line", "lines", "type", "place", "tool"],
    tokenHelp: {
      action: "Editing, Building, Playtesting…",
      script: "Script name",
      line: "Cursor line",
      lines: "Total lines",
      type: "server / local / module script",
      place: "Place name",
      tool: "Animation or VFX plugin",
    },
    styleHints: {
      classic: "Script, line and place: the classic look",
      minimal: "What you’re doing, nothing more",
      detailed: "Script, line and place, spelled out",
    },
    privacy: {
      script: ["Hide script names", "“DataStoreHandler” becomes “a script”."],
      place: ["Hide the place name", "Your game’s name stays off your profile."],
      lines: ["Hide line numbers", "Drops “Line 42 / 310” wherever it appears."],
      tools: ["Hide tool names", "{tool} stays empty, so nobody sees which animation or VFX plugin you use."],
      list: ["Places to keep ", "private", "While you’re in one of these, nothing is shared at all. Handy for commissions and secret projects.", "Place name, exactly as in Studio", "No private places yet."],
    },
    testing: ["Show playtests", "Off: playtesting keeps showing what you were editing before."],
    connection: {
      what: "Plugin",
      text: "UltimatePresence’s plugin for Roblox Studio.",
      empty: "Open a place in Roblox Studio with the plugin and it shows up here.",
      waiting: "Roblox Studio is open, waiting for the plugin",
      idle: "Roblox Studio isn’t running",
    },
    support: [
      ["Editing scripts", "Script name, line and script type", "DataStoreHandler (Line 42 / 310)", "ok"],
      ["Building", "When no script is open", "Developing", "ok"],
      ["Playtesting", "Play, Run and Play Here", "Testing", "ok"],
      ["Moon Animator 2", "Works with either plugin", "Animating", "ok"],
      ["Eclipse Animator", "Needs the UltimatePresence plugin", "Animating", "ok"],
      ["Qwinkle’s Part-Icles", "Needs the UltimatePresence plugin", "Making VFX", "ok"],
      ["Idle", "After 5 minutes without changes", "Idle", "ok"],
    ],
    // Images this app's own Discord application needs (Rich Presence → Art Assets).
    images: [
      ["studio", "Badge: Roblox Studio’s icon"],
      ["serverscript", "Editing a Script"],
      ["clientscript", "Editing a LocalScript"],
      ["modulescript", "Editing a ModuleScript"],
      ["icon_building", "Building"],
      ["icon_playtest", "Playtesting"],
      ["icon_animating", "Animating"],
      ["icon_vfx", "Making VFX"],
      ["icon_combo", "Animating + VFX"],
      ["icon_idle", "Idle"],
    ],
    iconsFolder: "icons-folder:roblox-studio",
    appName: "Roblox Studio",
  },
  blender: {
    glyph: "mesh",
    blurb: "Modeling, sculpting, UVs, shading, rigging, renders and more.",
    kinds: {
      modeling: "Modeling",
      sculpting: "Sculpting",
      uv: "UV unwrapping",
      texturing: "Texture painting",
      shading: "Shading",
      animating: "Animating",
      rigging: "Rigging",
      rendering: "Rendering",
      compositing: "Compositing",
      geonodes: "Geometry nodes",
      drawing: "Drawing",
      scripting: "Scripting",
      video: "Video editing",
      idle: "Idle",
    },
    tokens: ["action", "object", "file", "mode", "workspace", "frame", "frames", "progress", "version"],
    tokenHelp: {
      action: "Modeling, Sculpting, Rendering…",
      object: "Active object’s name",
      file: ".blend file name",
      mode: "Edit mode, Sculpt mode…",
      workspace: "Workspace tab name",
      frame: "Current frame (render frame while rendering)",
      frames: "Frames in the render",
      progress: "Render progress in %",
      version: "Blender version",
    },
    styleHints: {
      classic: "What you’re doing and the file",
      minimal: "Just what you’re doing",
      detailed: "Object, file, mode and version",
    },
    privacy: {
      script: ["Hide object names", "“Hull” disappears from “Sculpting Hull”."],
      place: ["Hide the file name", "Your .blend file’s name stays off your profile."],
      lines: ["Hide frame numbers", "Drops frame counts from renders and animation."],
      tools: null,
      list: ["Files to keep ", "private", "While one of these .blend files is open, nothing is shared at all.", "File name, e.g. client_project.blend", "No private files yet."],
    },
    testing: ["Show renders", "Off: while rendering, it keeps showing what you were doing before."],
    connection: {
      what: "Add-on",
      text: "UltimatePresence’s script for Blender.",
      empty: "Open Blender (with the script installed) and it shows up here.",
      waiting: "Blender is open, waiting for the script",
      idle: "Blender isn’t running",
    },
    support: [
      ["Modeling", "Object and Edit mode", "Modeling Hull", "ok"],
      ["Sculpting", "Sculpt mode", "Sculpting Hull", "ok"],
      ["UV unwrapping", "UV Editing workspace, or a UV editor while editing", "UV unwrapping Hull", "ok"],
      ["Texture & vertex painting", "Paint modes", "Texture painting Hull", "ok"],
      ["Shading", "Shading workspace", "Shading Hull", "ok"],
      ["Animating & rigging", "Animation workspace, timeline playing, Pose mode", "Animating", "ok"],
      ["Rendering", "While a render runs, with frame progress", "Rendering · Frame 120 of 250", "ok"],
      ["Compositing, geometry nodes, scripting, video", "Their workspaces and editors", "Compositing", "ok"],
      ["Grease Pencil", "Draw and edit modes", "Drawing", "ok"],
      ["Idle", "After 5 minutes without changes", "Idle", "ok"],
    ],
    images: [
      ["bl_blender", "Badge, plus scripting and video editing"],
      ["bl_modeling", "Modeling"],
      ["bl_sculpting", "Sculpting"],
      ["bl_uv", "UV unwrapping"],
      ["bl_painting", "Texture painting and drawing"],
      ["bl_shading", "Shading"],
      ["bl_animating", "Animating and rigging"],
      ["bl_rendering", "Rendering"],
      ["bl_nodes", "Compositing and geometry nodes"],
      ["icon_idle", "Idle"],
    ],
    iconsFolder: "icons-folder:blender",
    appName: "Blender",
  },
  "substance-painter": {
    glyph: "brush",
    blurb: "The project you’re texturing, the texture set and your bakes.",
    lede: ["Show the material you’re ", "painting", ", not just that Painter is open."],
    how: "A small Painter plugin (Painter has a built-in Python plugin system) will send updates to UltimatePresence, the same way the Blender add-on does.",
    example: { app: "Substance Painter", details: "Texturing Hull", state: "spaceship_v3 · Texture set: Metal", art: "brush" },
    planned: {
      Display: [
        ["Show project name", "For example “spaceship_v3”"],
        ["Show texture set", "The texture set you’re painting on"],
        ["Show bakes and exports", "“Baking” and “Exporting textures” while they run"],
        ["Custom wording", "The same template editor as the other apps"],
      ],
      Privacy: [["Hide project names", "Shows “a private project” instead"]],
      Behaviour: [["Idle after 5 minutes", "Like the other apps"]],
    },
  },
  figma: {
    glyph: "pen",
    blurb: "The file you’re designing in, and whether you’re editing or presenting.",
    lede: ["Let people know you’re ", "designing", ", or keep the file to yourself."],
    how: "Will read the Figma desktop app’s window title to see which file is open, so there’s nothing to install inside Figma.",
    example: { app: "Figma", details: "Designing Onboarding flow v2", state: "Page: Sign up", art: "pen" },
    planned: {
      Display: [
        ["Show file name", "For example “Onboarding flow v2”"],
        ["Show page name", "The page you’re working on inside the file"],
        ["Show when presenting", "“Presenting a prototype” instead of the file name"],
        ["Custom wording", "The same template editor as the other apps"],
      ],
      Privacy: [
        ["Hide file names", "Shows “a design file” instead"],
        ["Hide drafts", "Never share files that are still in Drafts"],
      ],
      Behaviour: [["Idle after 5 minutes", "Like the other apps"]],
    },
  },
};
const meta = (id) => APPS[id] || { glyph: "now", blurb: "", kinds: {} };
const kindsOf = (id) => meta(id).kinds || {};

/* ======================================================================
   Helpers
   ==================================================================== */

const SVG_NS = "http://www.w3.org/2000/svg";

/** Discord attachment links expire, and Discord won’t show them in Rich Presence. */
const isDiscordAttachment = (v) => /^https?:\/\/(cdn\.discordapp\.com|media\.discordapp\.net)\/attachments\//i.test(v || "");

function h(tag, props, ...kids) {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(props || {})) {
    if (v == null || v === false) continue;
    if (k === "class") el.className = v;
    else if (k === "text") el.textContent = v;
    else if (k === "dataset") Object.assign(el.dataset, v);
    else if (k.startsWith("on") && typeof v === "function") el.addEventListener(k.slice(2), v);
    else if (k in el && typeof v !== "string") el[k] = v;
    else el.setAttribute(k, v === true ? "" : v);
  }
  for (const kid of kids.flat(Infinity)) {
    if (kid == null || kid === false) continue;
    el.append(kid instanceof Node ? kid : document.createTextNode(String(kid)));
  }
  return el;
}

function icon(name, cls = "icon") {
  const svg = document.createElementNS(SVG_NS, "svg");
  svg.setAttribute("class", cls);
  svg.setAttribute("aria-hidden", "true");
  const use = document.createElementNS(SVG_NS, "use");
  use.setAttribute("href", `#i-${name}`);
  svg.append(use);
  return svg;
}
const setIcon = (svg, name) => svg.querySelector("use").setAttribute("href", `#i-${name}`);

const store = {
  get(k) { try { return localStorage.getItem(k); } catch { return null; } },
  set(k, v) { try { localStorage.setItem(k, v); } catch { /* not persisted */ } },
};

function debounce(fn, ms) {
  let t;
  return (...a) => { clearTimeout(t); t = setTimeout(() => fn(...a), ms); };
}

let toastTimer;
function toast(text, tone = "") {
  document.querySelector(".toast")?.remove();
  const t = h("div", { class: "toast", role: "status", dataset: { tone }, text });
  document.body.append(t);
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => t.remove(), 3200);
}

const timeFmt = new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" });
const timeFmtSec = new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit", second: "2-digit" });

/* ======================================================================
   Discord application info (name + uploaded art) for previews.
   ==================================================================== */

const appInfoCache = new Map();
function loadAppInfo(appId, { fresh = false } = {}) {
  if (!appId) return Promise.resolve(null);
  if (fresh) appInfoCache.delete(appId);
  if (!appInfoCache.has(appId)) {
    const api = "https://discord.com/api/v10";
    const json = (u) => fetch(u).then((r) => (r.ok ? r.json() : null)).catch(() => null);
    const p = Promise.all([json(`${api}/applications/${appId}/rpc`), json(`${api}/oauth2/applications/${appId}/assets`)])
      .then(([rpc, assets]) => {
        const info = { name: rpc && rpc.name ? rpc.name : null, assets: new Map((Array.isArray(assets) ? assets : []).map((a) => [a.name, a.id])) };
        if (!info.name && info.assets.size === 0) appInfoCache.delete(appId);
        return info;
      });
    appInfoCache.set(appId, p);
  }
  return appInfoCache.get(appId);
}
function assetUrl(appId, info, key) {
  if (!key) return null;
  if (/^https?:\/\//i.test(key)) return key;
  if (key.startsWith("mp:")) return `https://media.discordapp.net/${key.slice(3)}`;
  const id = info && info.assets.get(key);
  return id ? `https://cdn.discordapp.com/app-assets/${appId}/${id}.png?size=256` : null;
}

/* ======================================================================
   Components
   ==================================================================== */

function formatElapsed(ms) {
  let s = Math.max(0, Math.floor((Date.now() - ms) / 1000));
  const hr = Math.floor(s / 3600); s -= hr * 3600;
  const m = Math.floor(s / 60); s -= m * 60;
  const p = (n) => String(n).padStart(2, "0");
  return `${hr ? `${hr}:${p(m)}` : p(m)}:${p(s)} elapsed`;
}

const timers = new Set();
setInterval(() => timers.forEach((t) => t()), 1000);

/** Discord-style activity block. */
function presence({ compact = false } = {}) {
  const img = h("img", { class: "art-img", alt: "", hidden: true, draggable: "false" });
  const fbIcon = icon("cube");
  const fallback = h("div", { class: "art-fallback" }, fbIcon);
  const small = h("img", { class: "art-small", alt: "", hidden: true, draggable: "false" });
  img.addEventListener("error", () => { img.hidden = true; fallback.hidden = false; });
  small.addEventListener("error", () => { small.hidden = true; });
  const app = h("div", { class: "presence-app" });
  const details = h("div", { class: "presence-line" });
  const state = h("div", { class: "presence-line" });
  const time = h("div", { class: "presence-time" });
  const buttons = h("div", { class: "presence-buttons" });
  const el = h("div", { class: compact ? "presence compact" : "presence" },
    h("div", { class: "art" }, img, fallback, small),
    h("div", { class: "presence-text" }, app, details, state, time));
  let start = null;
  let token = 0;
  const tick = () => { time.textContent = start ? formatElapsed(start) : ""; };
  timers.add(tick);

  async function set(activity, appId, sourceId, fallbackName) {
    details.textContent = activity.details || "";
    state.textContent = activity.state || "";
    start = activity.startTimestamp || null;
    tick();
    buttons.replaceChildren(...(activity.buttons || []).map((b) => h("span", { text: b.label, title: b.url })));
    setIcon(fbIcon, meta(sourceId).glyph);
    if (!app.textContent) app.textContent = fallbackName || "";
    const my = ++token;
    const info = await loadAppInfo(appId);
    if (my !== token) return;
    app.textContent = (info && info.name) || fallbackName || "Your app";
    const large = assetUrl(appId, info, activity.largeImage);
    if (large) { if (img.getAttribute("src") !== large) img.src = large; img.hidden = false; img.title = activity.largeText || ""; }
    else { img.hidden = true; img.removeAttribute("src"); }
    fallback.hidden = !!large;
    const sm = assetUrl(appId, info, activity.smallImage);
    if (sm) { if (small.getAttribute("src") !== sm) small.src = sm; small.hidden = false; small.title = activity.smallText || ""; }
    else { small.hidden = true; small.removeAttribute("src"); }
  }
  return { el, set, buttons };
}

/** Your Discord profile picture, or your initial while it loads / without Discord. */
function paintAvatar(el, s) {
  const url = s.discordStatus === "connected" ? s.discordAvatar : null;
  el.dataset.online = String(s.discordStatus === "connected");
  if (url) {
    el.style.backgroundImage = `url("${url}")`;
    el.dataset.photo = "true";
    el.textContent = "";
  } else {
    el.style.backgroundImage = "";
    el.dataset.photo = "false";
    el.textContent = s.discordUser ? s.discordUser.trim().charAt(0).toUpperCase() : "?";
  }
}

/* Plugin / script setup ------------------------------------------------------ */

let integrations = null;
async function refreshIntegrations() {
  try { integrations = await invoke("get_integrations"); } catch { integrations = null; }
  document.dispatchEvent(new Event("integrations"));
  return integrations;
}

const INTEGRATIONS = {
  "roblox-studio": {
    key: "studioPlugin", which: "studio-plugin", name: "Roblox Studio plugin",
    status: { installed: "Installed", outdated: "Update ready", original: "Old StudioPresence plugin", missing: "Not installed" },
    action: { outdated: "Update plugin", original: "Switch to our plugin", missing: "Install plugin" },
    hint: { installed: "Restart Roblox Studio after installing or updating.", original: "The old plugin works, but can’t see Eclipse Animator or Part-Icles.", missing: "Adds the plugin to Roblox Studio. Restart Studio afterwards." },
  },
  blender: {
    key: "blenderAddon", which: "blender", name: "Blender script",
    status: { installed: "Installed", outdated: "Update ready", missing: "Not installed", "no-blender": "Blender not found" },
    action: { outdated: "Update script", missing: "Install script" },
    hint: { installed: "Runs by itself whenever Blender starts.", missing: "Runs by itself whenever Blender starts, nothing to enable. Restart Blender afterwards.", "no-blender": "Open Blender once (so it creates its settings), then come back." },
  },
};

/** Install / update button with the current state, for one app. */
function integrationWidget(id) {
  const cfg = INTEGRATIONS[id];
  const badge = h("span", { class: "badge" });
  const btn = h("button", { class: "btn primary", type: "button", hidden: true });
  const hint = h("span", { class: "setting-hint" });
  function paint() {
    const st = integrations ? integrations[cfg.key] : null;
    badge.textContent = st ? cfg.status[st] : "Checking…";
    badge.className = `badge ${st === "installed" ? "ok" : "next"}`;
    btn.hidden = !(st && cfg.action[st]);
    btn.textContent = (st && cfg.action[st]) || "";
    let text = (st && cfg.hint[st]) || "";
    if (st === "installed" && id === "blender" && integrations.blenderVersions.length) text = `For Blender ${integrations.blenderVersions.join(", ")}. ${text}`;
    hint.textContent = text;
  }
  btn.addEventListener("click", async () => {
    btn.disabled = true;
    try { toast(await invoke("install_integration", { which: cfg.which })); }
    catch (e) { toast(String(e), "bad"); }
    btn.disabled = false;
    await refreshIntegrations();
  });
  document.addEventListener("integrations", paint);
  paint();
  return h("div", { class: "integration" }, h("div", { class: "integration-row" }, badge, btn), hint);
}

function tile(id, { size = "", live = false, soon = false } = {}) {
  return h("span", { class: `tile ${size}`.trim(), dataset: { live: String(live), soon: String(soon) } }, icon(meta(id).glyph));
}

function sw({ checked = false, disabled = false, label, big = false, onchange }) {
  return h("input", {
    type: "checkbox", class: big ? "switch lg" : "switch", role: "switch",
    checked, disabled, "aria-label": label,
    onchange: onchange && ((e) => onchange(e.target.checked, e.target)),
  });
}

/** A whole-row switch setting. */
function switchSetting(title, hint, input) {
  return h("label", { class: "setting", dataset: { disabled: String(input.disabled) } },
    h("span", { class: "setting-text" }, h("span", { class: "setting-title", text: title }), hint && h("span", { class: "setting-hint", text: hint })),
    input);
}

function setting(title, hint, control, { stack = false } = {}) {
  return h("div", { class: stack ? "setting stack" : "setting" },
    h("div", { class: "setting-text" }, h("span", { class: "setting-title", text: title }), hint && h("span", { class: "setting-hint", text: hint })),
    control);
}

function segmented(options, value, onpick, { label, disabled = false } = {}) {
  const btns = options.map(([v, text]) => h("button", { type: "button", "aria-pressed": String(v === value), text, disabled }));
  btns.forEach((b, i) => b.addEventListener("click", () => {
    btns.forEach((x) => x.setAttribute("aria-pressed", String(x === b)));
    onpick(options[i][0]);
  }));
  const el = h("div", { class: "segmented", role: "group", "aria-label": label }, btns);
  el.setValue = (v) => btns.forEach((b, i) => b.setAttribute("aria-pressed", String(options[i][0] === v)));
  return el;
}

function sectionHead(title, lede, accent) {
  return [
    h("h2", { class: "section-title" }, title, accent && h("em", { text: accent })),
    lede && h("p", { class: "section-lede", text: lede }),
  ];
}

/* ======================================================================
   State
   ==================================================================== */

let snap = null;
let route = store.get("route") || "home";
let page = null;
const logEntries = [];
let unseenProblems = 0;

const isLive = (s) => !!(s && s.shown && s.discordStatus === "connected");
const src = (id) => snap && snap.sources.find((s) => s.id === id);

async function patchSource(id, patch) {
  try {
    await invoke("update_source_settings", { source: id, patch });
    return true;
  } catch (e) {
    toast(String(e), "bad");
    return false;
  }
}

/* ======================================================================
   Sidebar
   ==================================================================== */

const navApps = document.getElementById("nav-apps");
const navItems = new Map();

function buildNav() {
  navApps.replaceChildren();
  navItems.clear();
  for (const s of snap.sources) {
    const t = tile(s.id, { soon: !s.available });
    const sub = h("span", { class: "nav-sub" });
    const dot = h("span", { class: "dot" });
    const btn = h("button", { class: "nav-item", type: "button", dataset: { route: `app:${s.id}` }, title: s.name },
      t, h("span", { class: "nav-text" }, h("span", { class: "nav-name", text: s.name }), sub), dot);
    btn.addEventListener("click", () => go(`app:${s.id}`));
    navApps.append(btn);
    navItems.set(s.id, { t, sub, dot });
  }
  markRoute();
}

function updateNav() {
  if (navItems.size !== snap.sources.length) buildNav();
  for (const s of snap.sources) {
    const n = navItems.get(s.id);
    n.sub.textContent = s.statusText;
    n.dot.dataset.s = s.available ? s.status : "soon";
    n.t.dataset.live = String(snap.activeSource === s.id && isLive(snap));
  }
  const live = isLive(snap);
  paintAvatar(document.getElementById("me-avatar"), snap);
  document.getElementById("me-name").textContent = snap.discordUser || "Discord";
  const state = document.getElementById("me-state");
  const active = snap.activeSource ? src(snap.activeSource) : null;
  state.textContent = snap.discordStatus !== "connected" ? (snap.discordStatus === "rejected" ? "App ID rejected" : "Discord isn’t running")
    : snap.paused ? "Paused" : live ? `Live · ${active ? active.name : ""}` : "Online, nothing shared";
  state.dataset.tone = live ? "live" : "";
  const pauseBtn = document.getElementById("me-pause");
  setIcon(pauseBtn.querySelector("svg"), snap.paused ? "play" : "pause");
  pauseBtn.title = pauseBtn.ariaLabel = snap.paused ? "Resume sharing" : "Pause sharing";

  const count = document.getElementById("nav-log-count");
  count.hidden = unseenProblems === 0 || route === "log";
  count.textContent = String(unseenProblems);
}

function markRoute() {
  const base = route.split("/")[0];
  document.querySelectorAll(".sidebar [data-route]").forEach((b) => {
    if (b.dataset.route === base) b.setAttribute("aria-current", "page");
    else b.removeAttribute("aria-current");
  });
}

/* ======================================================================
   Router
   ==================================================================== */

const main = document.getElementById("main");

function go(next) {
  const [base, tab] = next.split("/");
  if (base.startsWith("app:") && !src(base.slice(4))) next = "home";
  route = next;
  store.set("route", route.split("/")[0]);
  timers.clear();
  if (route === "log") unseenProblems = 0;
  const id = base.slice(4);
  page = base === "home" ? homePage()
    : base === "settings" ? settingsPage()
    : base === "log" ? logPage()
    : src(id).available ? livePage(id, tab)
    : plannedPage(id);
  main.replaceChildren(page.el);
  main.scrollTop = 0;
  page.update(snap);
  markRoute();
  updateNav();
}

/* ======================================================================
   Home
   ==================================================================== */

function headline(s) {
  const active = s.activeSource ? src(s.activeSource) : null;
  const studio = s.sources.find((x) => x.available) || {};
  if (s.discordStatus === "rejected") return ["Discord ", "rejected", " the app.", "The Discord Application ID isn’t valid. Check it on the app’s page, or switch back to the default."];
  if (s.paused) return ["Sharing is ", "paused", ".", "Your Discord status is clear. Resume whenever you’re ready."];
  if (s.shown && s.discordStatus !== "connected") return ["Ready when ", "Discord", " is.", `Open Discord and ${active ? active.name : "your app"} will appear on your profile straight away.`];
  if (s.shown) return ["You’re ", "live", " on Discord.", `Everyone who can see your activity sees ${active ? active.name : "this"}, exactly as below.`];
  if (studio.status === "error") return ["Something’s ", "blocking", " Studio.", studio.detail || "Port 4455 is in use by another program."];
  if (studio.status === "held") return ["Keeping it ", "quiet", ".", `${studio.detail}. Your settings are holding your status back.`];
  if (studio.status === "waiting") return ["Waiting for ", "the plugin", ".", "Roblox Studio is open, but the plugin hasn’t said anything yet. Open a place, and check the plugin is installed (see Get set up below)."];
  if (s.discordStatus !== "connected") return ["Waiting for ", "Discord", ".", "Open Discord and UltimatePresence will connect by itself."];
  return ["Nothing to share ", "yet", ".", "Open Roblox Studio or Blender and your work shows up here, and on your profile, as you go. Whichever app you have in front wins."];
}

function homePage() {
  const h1 = h("h1");
  const lede = h("p", { class: "lede" });

  const avatar = h("span", { class: "avatar" });
  const name = h("strong");
  const handle = h("span");
  const pres = presence();
  const emptyIcon = icon("now");
  const emptyTitle = h("p", { class: "empty-title" });
  const emptyText = h("p", { class: "empty-text" });
  const emptyBtn = h("button", { class: "btn primary", type: "button", hidden: true });
  const empty = h("div", { class: "empty" }, h("div", { class: "empty-mark" }, emptyIcon), h("div", {}, emptyTitle, emptyText, emptyBtn));
  const activity = h("div", { class: "activity" }, h("div", { class: "activity-label", text: "Playing" }), pres.el, pres.buttons, empty);
  const noteText = h("span");
  const note = h("p", { class: "note", hidden: true }, icon("alert"), noteText);
  const pauseBtn = h("button", { class: "btn", type: "button" });
  const customise = h("button", { class: "btn ghost", type: "button" }, "Customise", icon("chevron"));
  const profile = h("section", { class: "profile", "aria-label": "Your Discord profile" },
    h("div", { class: "profile-banner" }),
    h("div", { class: "profile-head" }, avatar),
    h("div", { class: "profile-name" }, name, handle),
    activity, note,
    h("div", { class: "profile-foot" }, pauseBtn, customise));

  const since = h("span");
  const timeline = h("ol", { class: "timeline" });
  const session = h("section", { class: "session" },
    h("div", { class: "session-head" }, h("h2", { text: "This session" }), since),
    timeline);

  const apps = h("div", { class: "apps" });
  const rows = new Map();

  let emptyAction = null;
  emptyBtn.addEventListener("click", () => emptyAction && emptyAction());
  pauseBtn.addEventListener("click", () => invoke("set_paused", { paused: !snap.paused }));
  customise.addEventListener("click", () => go(`app:${snap.activeSource || "roblox-studio"}/display`));

  const updateText = h("strong");
  const updateBtn = h("button", { class: "btn primary", type: "button", text: "Install and restart" });
  updateBtn.addEventListener("click", installUpdate(updateBtn));
  const updateBanner = h("div", { class: "callout info", hidden: true },
    icon("sparkle"), h("div", { style: "display:flex;align-items:center;gap:16px;flex-wrap:wrap;flex:1" }, h("div", { style: "flex:1" }, updateText, h("p", { class: "setting-hint", text: "Takes a few seconds. Your settings stay as they are." })), updateBtn));

  const discordRow = h("span", { class: "badge" });
  const setupRows = [
    h("div", { class: "setting" }, h("div", { class: "setting-text" }, h("span", { class: "setting-title", text: "Discord" }), h("span", { class: "setting-hint", text: "The Discord desktop app needs to be open. UltimatePresence connects by itself." })), discordRow),
    ...Object.keys(INTEGRATIONS).map((id) => h("div", { class: "setting" },
      h("div", { class: "setting-text" }, h("span", { class: "setting-title", text: INTEGRATIONS[id].name })),
      integrationWidget(id))),
  ];
  const setup = h("section", { class: "section" }, sectionHead("Get set ", "Three things, then it runs by itself. You can always come back here.", "up"), h("div", { class: "group" }, setupRows));

  const el = h("div", { class: "page" },
    updateBanner,
    h("header", { class: "hero" }, h("div", {}, h1, lede)),
    h("div", { class: "home-grid" }, profile, session),
    setup,
    h("section", { class: "section" }, sectionHead("Your ", "Each app has its own page with its own settings.", "apps"), apps));

  const paintSetup = () => {
    const done = integrations && integrations.studioPlugin === "installed" && integrations.blenderAddon === "installed" && snap.discordStatus === "connected";
    setup.hidden = !!done;
  };
  document.addEventListener("integrations", paintSetup);

  function update(s) {
    updateBanner.hidden = !s.updateAvailable;
    updateText.textContent = `UltimatePresence ${s.updateAvailable} is available`;
    const discordOk = s.discordStatus === "connected";
    discordRow.className = `badge ${discordOk ? "ok" : "next"}`;
    discordRow.textContent = discordOk ? "Connected" : "Not running";
    paintSetup();
    const [a, b, c, l] = headline(s);
    h1.replaceChildren(a, h("em", { text: b }), c);
    lede.textContent = l;

    const live = isLive(s);
    profile.dataset.live = String(live);
    const connected = s.discordStatus === "connected";
    paintAvatar(avatar, s);
    name.textContent = s.discordUser || "Your profile";
    handle.textContent = connected ? "Online in Discord" : "Discord isn’t connected";
    pauseBtn.replaceChildren(icon(s.paused ? "play" : "pause"), s.paused ? "Resume sharing" : "Pause sharing");

    pres.el.hidden = !s.shown;
    pres.buttons.hidden = !s.shown;
    empty.hidden = !!s.shown;
    if (s.shown) {
      const active = src(s.activeSource);
      pres.set(s.shown, s.discordApplicationId, s.activeSource, active && active.name);
    } else {
      emptyBtn.hidden = true;
      emptyAction = null;
      const studio = s.sources.find((x) => x.available) || {};
      if (s.paused) {
        setIcon(emptyIcon, "pause");
        emptyTitle.textContent = "Nothing while paused";
        emptyText.textContent = "Your profile shows no activity until you resume.";
        emptyBtn.hidden = false; emptyBtn.textContent = "Resume sharing";
        emptyAction = () => invoke("set_paused", { paused: false });
      } else if (studio.status === "held") {
        setIcon(emptyIcon, "moon");
        emptyTitle.textContent = "Held back";
        emptyText.textContent = `${studio.detail}.`;
        emptyBtn.hidden = false; emptyBtn.textContent = "Change this";
        emptyAction = () => go(`app:${studio.id}/${/place/i.test(studio.detail) ? "privacy" : "behaviour"}`);
      } else if (studio.status === "disabled") {
        setIcon(emptyIcon, "cube");
        emptyTitle.textContent = `${studio.name} is turned off`;
        emptyText.textContent = "Turn it back on to share what you’re working on.";
        emptyBtn.hidden = false; emptyBtn.textContent = `Turn on ${studio.name}`;
        emptyAction = () => patchSource(studio.id, { enabled: true });
      } else {
        setIcon(emptyIcon, studio.status === "error" ? "alert" : "now");
        emptyTitle.textContent = studio.status === "waiting" ? "Studio’s open" : "Nothing yet";
        emptyText.textContent = studio.status === "waiting" ? "Waiting for the plugin to send its first update." : "Your activity appears here the moment you open Roblox Studio or Blender.";
      }
    }

    if (s.discordStatus === "rejected") { note.hidden = false; noteText.textContent = `Discord said: ${s.discordMessage || "invalid Application ID"}.`; }
    else if (!connected && s.shown) { note.hidden = false; noteText.textContent = "Not visible yet: Discord isn’t running. It’ll appear the moment Discord opens."; }
    else note.hidden = true;

    since.textContent = `since ${timeFmt.format(new Date(s.startedMs))}`;
    const items = s.history.map((it) => h("li", { dataset: { stop: String(!it.source) } },
      h("time", { text: timeFmt.format(new Date(it.timeMs)) }),
      h("span", { class: "t-dot" }),
      h("div", {}, h("div", { class: "t-title", text: it.title }), it.subtitle && h("div", { class: "t-sub", text: it.subtitle }))));
    timeline.replaceChildren(...(items.length ? items : [h("li", { class: "timeline-empty", text: "Your session fills in here as you work: scripts, playtests, tools." })]));

    for (const x of s.sources) {
      let r = rows.get(x.id);
      if (!r) {
        const t = tile(x.id, { soon: !x.available });
        const dot = h("span", { class: "dot" });
        const status = h("span");
        const toggle = x.available
          ? sw({ label: `Show ${x.name} on Discord`, onchange: (on) => patchSource(x.id, { enabled: on }) })
          : null;
        const row = h("div", { class: "app-row", role: "link", tabindex: "0", "aria-label": `${x.name} settings` },
          t,
          h("div", { class: "app-main" }, h("div", { class: "app-name", text: x.name }), h("div", { class: "app-blurb", text: meta(x.id).blurb })),
          h("div", { class: "app-end" },
            x.available ? h("span", { class: "app-status" }, dot, status) : h("span", { class: "chip soon", text: "Coming soon" }),
            toggle, icon("chevron")));
        row.addEventListener("click", (e) => { if (e.target !== toggle) go(`app:${x.id}`); });
        row.addEventListener("keydown", (e) => { if (e.key === "Enter") go(`app:${x.id}`); });
        toggle && toggle.addEventListener("click", (e) => e.stopPropagation());
        apps.append(row);
        r = { t, dot, status, toggle };
        rows.set(x.id, r);
      }
      r.t.dataset.live = String(s.activeSource === x.id && live);
      r.dot.dataset.s = x.status;
      r.status.textContent = x.statusText;
      if (r.toggle) r.toggle.checked = x.settings.enabled;
    }
  }
  return { el, update };
}

/* ======================================================================
   Live app page (Roblox Studio today)
   ==================================================================== */

const TABS = [["overview", "Overview"], ["display", "Display"], ["privacy", "Privacy"], ["behaviour", "Behaviour"]];

/** Built-in wording per app (style → state → { top, bottom }), loaded once. */
const presetCache = {};
function getPresets(id) {
  if (!presetCache[id]) presetCache[id] = invoke("get_presets", { source: id }).catch(() => null);
  return presetCache[id];
}

const firstKind = (id) => Object.keys(kindsOf(id))[0];
const shortKind = (label) => label.replace(" a script", "");

function livePage(id, initialTab) {
  const m = meta(id);
  const s0 = src(id);
  const saved = store.get(`tab:${id}`);
  let tab = TABS.some(([t]) => t === initialTab) ? initialTab : TABS.some(([t]) => t === saved) ? saved : "overview";

  const heroTile = tile(id, { size: "xl" });
  const statusRow = h("div", { class: "status-row" });
  const enabled = sw({ big: true, label: `Show ${s0.name} on Discord`, onchange: (on) => patchSource(id, { enabled: on }) });
  const toggle = h("label", { class: "big-toggle" }, "Show on Discord", enabled);

  const tabBtns = TABS.map(([t, label]) => h("button", { type: "button", role: "tab", "aria-selected": String(t === tab), text: label }));
  const indicator = h("span", { class: "indicator" });
  const tabs = h("div", { class: "tabs", role: "tablist" }, tabBtns, indicator);
  const body = h("div");

  // Shared live preview
  const pv = presence({ compact: true });
  const pvSub = h("p", { class: "sub" });
  let sample = null; // null = live
  const sampleBtns = [["", "Live"], ...Object.entries(kindsOf(id))].map(([k, label]) =>
    h("button", { type: "button", "aria-pressed": String(k === ""), text: k ? shortKind(label) : "Live", dataset: { k } }));
  sampleBtns.forEach((b) => b.addEventListener("click", () => {
    sample = b.dataset.k || null;
    sampleBtns.forEach((x) => x.setAttribute("aria-pressed", String(x === b)));
    refreshPreview();
  }));
  const previewCard = h("aside", { class: "preview-card" },
    h("h3", { text: "Preview" }), pvSub,
    h("div", { class: "activity" }, h("div", { class: "activity-label", text: "Playing" }), pv.el, pv.buttons),
    h("div", { class: "sample-chips", role: "group", "aria-label": "Preview state" }, sampleBtns));

  const refreshPreview = debounce(async () => {
    const live = src(id).activity;
    const kind = sample || (live ? null : firstKind(id));
    sampleBtns[0].disabled = !live;
    if (!live && !sample) sampleBtns.forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.k === kind)));
    try {
      const a = await invoke("render_preview", { source: id, patch: null, kind });
      pv.set(a, src(id).settings.applicationId, id, s0.name);
      pvSub.textContent = kind ? `Sample: ${kindsOf(id)[kind].toLowerCase()}, with your settings.` : `Live from ${s0.name}, exactly as Discord shows it.`;
    } catch { /* ignore */ }
  }, 120);

  let current = null; // { el, update }
  function showTab(t, focus) {
    tab = t;
    store.set(`tab:${id}`, t);
    tabBtns.forEach((b, i) => b.setAttribute("aria-selected", String(TABS[i][0] === t)));
    requestAnimationFrame(placeIndicator);
    current = { overview: overviewTab, display: displayTab, privacy: privacyTab, behaviour: behaviourTab }[t](id);
    const withPreview = t !== "overview";
    body.replaceChildren(withPreview
      ? h("div", { class: "split" }, h("div", { class: "stack" }, current.el), h("div", { class: "preview-col" }, h("div", { class: "sticky" }, previewCard)))
      : current.el);
    current.update(snap);
    if (withPreview) refreshPreview();
    if (focus) tabBtns[TABS.findIndex(([x]) => x === t)].focus();
  }
  function placeIndicator() {
    const b = tabBtns[TABS.findIndex(([x]) => x === tab)];
    indicator.style.width = `${b.offsetWidth}px`;
    indicator.style.transform = `translateX(${b.offsetLeft}px)`;
  }
  tabBtns.forEach((b, i) => {
    b.addEventListener("click", () => showTab(TABS[i][0]));
    b.addEventListener("keydown", (e) => {
      const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
      if (d) showTab(TABS[(i + d + TABS.length) % TABS.length][0], true);
    });
  });
  window.addEventListener("resize", placeIndicator);

  const el = h("div", { class: "page" },
    h("header", { class: "app-hero" },
      h("div", { class: "app-hero-main" }, heroTile, h("div", {}, h("h1", { text: s0.name }), statusRow)),
      toggle),
    tabs, body);

  showTab(tab);

  return {
    el,
    update(s) {
      const x = src(id);
      if (!x) return;
      const focused = s.focused === id ? " · focused" : "";
      statusRow.replaceChildren(h("span", { class: "dot", dataset: { s: x.status } }), h("span", { text: x.statusText + focused }));
      heroTile.dataset.live = String(s.activeSource === id && isLive(s));
      enabled.checked = x.settings.enabled;
      toggle.dataset.on = String(x.settings.enabled);
      current && current.update(s);
      if (tab !== "overview") refreshPreview();
    },
  };
}

function overviewTab(id) {
  const m = meta(id);
  const con = m.connection;
  const callout = h("div", { class: "callout", role: "alert", hidden: true }, icon("alert"), h("div"));
  const status = h("dd");
  const pres = presence();
  const nothing = h("div", { class: "empty" }, h("div", { class: "empty-mark" }, icon(m.glyph)),
    h("div", {}, h("p", { class: "empty-title", text: "Nothing coming in yet" }), h("p", { class: "empty-text", text: con.empty })));
  const showing = h("div", { class: "activity" }, h("div", { class: "activity-label", text: "Sending to Discord" }), pres.el, pres.buttons, nothing);

  const el = h("div", {},
    callout,
    h("div", { class: "home-grid" },
      h("section", { class: "profile" }, showing),
      h("section", { class: "session" },
        h("div", { class: "session-head" }, h("h2", { text: "Connection" })),
        h("dl", { class: "facts", style: "margin:0 -26px -18px" },
          h("dt", { text: "Status" }), status,
          h("dt", { text: con.what }), h("dd", {}, h("span", { text: con.text }), integrationWidget(id)),
          h("dt", { text: "Address" }), h("dd", {}, h("code", { text: "localhost:4455" }), h("br"), h("span", { class: "setting-hint", text: "This PC only, web pages are blocked" }))))),
    h("section", { class: "section" },
      sectionHead("What it ", "Detected automatically while you work. The app you have in front wins when several are open.", "picks up"),
      h("div", { class: "group" }, h("ul", { class: "support" }, m.support.map(([what, how, eg, st]) => h("li", {},
        h("div", { class: "what" }, h("strong", { text: what }), h("span", { text: how })),
        h("span", { class: "eg", text: eg }),
        h("span", { class: `badge ${st}`, text: st === "ok" ? "Supported" : "In progress" })))))));

  return {
    el,
    update() {
      const x = src(id);
      callout.hidden = !(x.status === "error" || x.status === "held");
      callout.className = x.status === "held" ? "callout info" : "callout";
      callout.lastChild.textContent = x.detail || "";
      status.textContent = {
        sharing: "Connected and sending updates",
        waiting: con.waiting,
        idle: con.idle,
        disabled: "Turned off, updates are ignored",
        held: x.detail,
        error: x.detail || "Can’t listen for updates",
      }[x.status] || x.statusText;
      pres.el.hidden = !x.activity;
      pres.buttons.hidden = !x.activity;
      nothing.hidden = !!x.activity;
      if (x.activity) pres.set(x.activity, x.settings.applicationId, id, x.name);
    },
  };
}

function displayTab(id) {
  const m = meta(id);
  const kinds = kindsOf(id);
  const STYLE_INFO = [
    ["classic", "Classic", m.styleHints.classic],
    ["minimal", "Minimal", m.styleHints.minimal],
    ["detailed", "Detailed", m.styleHints.detailed],
    ["custom", "Custom", "Your own wording for every state"],
  ];
  const samples = new Map();
  const options = STYLE_INFO.map(([v, name, hint]) => {
    const sampleEl = h("div", { class: "style-sample" }, h("span"), h("span"));
    samples.set(v, sampleEl);
    const b = h("button", { class: "style-option", type: "button", role: "radio", "aria-checked": "false" },
      h("span", { class: "style-name", text: name }), h("span", { class: "setting-hint", text: hint }), sampleEl);
    b.addEventListener("click", () => pickStyle(v));
    return b;
  });

  async function pickStyle(v) {
    const x = src(id);
    const patch = { style: v };
    const presets = await getPresets(id);
    // Starting a custom style from the current one, so there's something to edit.
    if (v === "custom" && Object.keys(x.settings.templates || {}).length === 0 && presets) {
      patch.templates = JSON.parse(JSON.stringify(presets.styles[x.settings.style] || presets.styles.classic));
    }
    await patchSource(id, patch);
  }

  const refreshSamples = debounce(async () => {
    for (const [v] of STYLE_INFO) {
      try {
        const a = await invoke("render_preview", { source: id, patch: { style: v }, kind: firstKind(id) });
        const [top, bottom] = samples.get(v).children;
        top.textContent = a.details || "";
        bottom.textContent = a.state || "";
      } catch { /* ignore */ }
    }
  }, 150);

  // Custom wording editor
  const inputs = {};
  let lastFocused = null;
  const save = debounce(() => {
    const templates = {};
    for (const k of Object.keys(kinds)) templates[k] = { top: inputs[k].top.value, bottom: inputs[k].bottom.value };
    patchSource(id, { templates });
  }, 400);
  const rows = Object.entries(kinds).flatMap(([k, label]) => {
    const mk = (part) => {
      const f = h("input", { class: "field", spellcheck: "false", "aria-label": `${label}, ${part} line`, title: `${part === "top" ? "Top" : "Bottom"} line`, maxlength: "128" });
      f.addEventListener("focus", () => (lastFocused = f));
      f.addEventListener("input", save);
      return f;
    };
    inputs[k] = { top: mk("top"), bottom: mk("bottom") };
    return [h("span", { class: "kind", text: label }), inputs[k].top, inputs[k].bottom];
  });
  const tokenBtns = m.tokens.map((t) => h("button", { class: "token", type: "button", title: m.tokenHelp[t], text: `{${t}}` }));
  tokenBtns.forEach((b) => {
    b.addEventListener("mousedown", (e) => e.preventDefault());
    b.addEventListener("click", () => {
      const f = lastFocused || inputs[firstKind(id)].top;
      const at = f.selectionStart ?? f.value.length;
      f.value = f.value.slice(0, at) + b.textContent + f.value.slice(f.selectionEnd ?? at);
      f.focus();
      f.setSelectionRange(at + b.textContent.length, at + b.textContent.length);
      save();
    });
  });
  const custom = h("div", { class: "group" },
    h("div", { class: "templates" }, rows),
    h("div", { class: "tokens" }, h("span", { class: "hint", text: "Insert:" }), tokenBtns),
    h("p", { class: "setting-hint", style: "padding:0 22px 20px" }, "Wrap text in [ ] to drop it when a value is hidden or missing, like “{action}[ {object}]”."));
  const customSection = h("section", {}, sectionHead("Your ", "Write the two lines Discord shows for each state.", "wording"), custom);

  const timer = segmented([["session", `Since ${m.appName} opened`], ["activity", "Current task"], ["off", "Off"]], "session",
    (v) => patchSource(id, { timer: v }), { label: "Timer" });

  const el = h("div", { class: "stack", style: "display:flex;flex-direction:column;gap:56px" },
    h("section", {}, sectionHead("Pick a ", "How your status reads on Discord. Every style respects your privacy settings.", "style"),
      h("div", { class: "styles", role: "radiogroup", "aria-label": "Style" }, options)),
    customSection,
    h("section", {}, sectionHead("The ", null, "timer"),
      h("div", { class: "group" }, setting("What the clock counts", "The “elapsed” time under your status.", timer, { stack: true }))));

  return {
    el,
    async update() {
      const x = src(id).settings;
      options.forEach((o, i) => o.setAttribute("aria-checked", String(STYLE_INFO[i][0] === x.style)));
      customSection.hidden = x.style !== "custom";
      const presets = await getPresets(id);
      for (const k of Object.keys(kinds)) {
        const t = (x.templates || {})[k] || {};
        const ph = presets && presets.styles.classic[k];
        for (const part of ["top", "bottom"]) {
          const f = inputs[k][part];
          if (document.activeElement !== f) f.value = t[part] || "";
          f.placeholder = ph ? ph[part] : "";
        }
      }
      timer.setValue(x.timer);
      refreshSamples();
    },
  };
}

function privacyTab(id) {
  const p = meta(id).privacy;
  const hideScript = sw({ label: p.script[0], onchange: (on) => patchSource(id, { hideScript: on }) });
  const hidePlace = sw({ label: p.place[0], onchange: (on) => patchSource(id, { hidePlace: on }) });
  const hideLines = sw({ label: p.lines[0], onchange: (on) => patchSource(id, { hideLines: on }) });
  const hideTools = p.tools && sw({ label: p.tools[0], onchange: (on) => patchSource(id, { hideTools: on }) });
  const alias = h("input", { class: "field", maxlength: "64", placeholder: "a private project", "aria-label": "Show this instead" });
  alias.addEventListener("input", debounce(() => patchSource(id, { placeAlias: alias.value }), 400));
  const aliasRow = setting("Show this instead", "What people see where the name would be.", alias, { stack: true });

  const [listA, listB, listLede, listPlaceholder, listEmpty] = p.list;
  const tags = h("div", { class: "tags" });
  const addInput = h("input", { class: "field", placeholder: listPlaceholder, "aria-label": "Add to the private list" });
  const addBtn = h("button", { class: "btn", type: "button", text: "Add" });
  const add = async () => {
    const v = addInput.value.trim();
    if (!v) return;
    const list = src(id).settings.hiddenPlaces;
    if (!list.some((q) => q.toLowerCase() === v.toLowerCase())) await patchSource(id, { hiddenPlaces: [...list, v] });
    addInput.value = "";
  };
  addBtn.addEventListener("click", add);
  addInput.addEventListener("keydown", (e) => { if (e.key === "Enter") add(); });

  const el = h("div", { style: "display:flex;flex-direction:column;gap:56px" },
    h("section", {}, sectionHead("What to ", "Hide the details and still show that you’re working.", "hide"),
      h("div", { class: "group" },
        switchSetting(p.script[0], p.script[1], hideScript),
        switchSetting(p.place[0], p.place[1], hidePlace),
        aliasRow,
        switchSetting(p.lines[0], p.lines[1], hideLines),
        hideTools && switchSetting(p.tools[0], p.tools[1], hideTools))),
    h("section", {}, sectionHead(listA, listLede, listB),
      h("div", { class: "group" }, h("div", { class: "setting stack" }, tags, h("div", { class: "tag-input" }, addInput, addBtn)))));

  return {
    el,
    update() {
      const x = src(id).settings;
      hideScript.checked = x.hideScript;
      hidePlace.checked = x.hidePlace;
      hideLines.checked = x.hideLines;
      if (hideTools) hideTools.checked = x.hideTools;
      aliasRow.hidden = !x.hidePlace;
      if (document.activeElement !== alias) alias.value = x.placeAlias;
      tags.replaceChildren(...(x.hiddenPlaces.length
        ? x.hiddenPlaces.map((q) => h("span", { class: "tag" }, q,
          h("button", { type: "button", "aria-label": `Remove ${q}`, onclick: () => patchSource(id, { hiddenPlaces: x.hiddenPlaces.filter((r) => r !== q) }) }, icon("x"))))
        : [h("span", { class: "setting-hint", text: listEmpty })]));
    },
  };
}

function behaviourTab(id) {
  const m = meta(id);
  const testing = sw({ label: m.testing[0], onchange: (on) => patchSource(id, { showTesting: on }) });

  const btnRows = [0, 1].map((i) => {
    const label = h("input", { class: "field", maxlength: "32", placeholder: i ? "Second button" : "My portfolio", "aria-label": `Button ${i + 1} label` });
    const url = h("input", { class: "field", placeholder: "https://…", "aria-label": `Button ${i + 1} link` });
    return { label, url, row: [h("span", { class: "n", text: String(i + 1) }), label, url] };
  });
  const btnError = h("p", { class: "error-text", hidden: true });
  const saveButtons = debounce(async () => {
    const buttons = btnRows.map((r) => ({ label: r.label.value.trim(), url: r.url.value.trim() })).filter((b) => b.label || b.url);
    const incomplete = buttons.find((b) => !b.label || !/^https?:\/\/\S+/.test(b.url));
    if (incomplete) {
      btnError.hidden = false;
      btnError.textContent = !incomplete.label ? "Give each button a label." : "Links need to start with https://";
      return;
    }
    btnError.hidden = true;
    try { await invoke("update_source_settings", { source: id, patch: { buttons } }); }
    catch (e) { btnError.hidden = false; btnError.textContent = String(e); }
  }, 600);
  btnRows.forEach((r) => { r.label.addEventListener("input", saveButtons); r.url.addEventListener("input", saveButtons); });

  const reset = h("button", { class: "btn danger", type: "button", text: "Reset settings" });
  let armed = 0;
  reset.addEventListener("click", async () => {
    if (Date.now() < armed) {
      armed = 0;
      reset.textContent = "Reset settings";
      await invoke("reset_source_settings", { source: id });
      toast("Settings reset. Your Application ID was kept.");
    } else {
      armed = Date.now() + 3000;
      reset.textContent = "Click again to reset";
      setTimeout(() => { if (Date.now() >= armed) reset.textContent = "Reset settings"; }, 3100);
    }
  });

  const el = h("div", { style: "display:flex;flex-direction:column;gap:56px" },
    h("section", {}, sectionHead("When to ", "After 5 minutes without changes your status switches to Idle, with the Zzz icon.", "share"),
      h("div", { class: "group" }, switchSetting(m.testing[0], m.testing[1], testing))),
    h("section", {}, sectionHead("Profile ", "Up to two buttons under your status. Others see them; you can’t click your own.", "buttons"),
      h("div", { class: "group" }, h("div", { class: "setting stack" },
        h("div", { class: "btn-editor" }, btnRows.flatMap((r) => r.row)), btnError))),
    h("section", {}, sectionHead("Start ", `Puts every ${m.appName} setting back to its default.`, "over"),
      h("div", { class: "group" }, setting(`Reset ${m.appName} settings`, "Your Discord Application ID stays as it is.", reset))));

  return {
    el,
    update() {
      const x = src(id).settings;
      testing.checked = x.showTesting;
      btnRows.forEach((r, i) => {
        const b = x.buttons[i] || { label: "", url: "" };
        if (document.activeElement !== r.label && document.activeElement !== r.url) {
          r.label.value = b.label;
          r.url.value = b.url;
        }
      });
    },
  };
}

/* ======================================================================
   Planned app page
   ==================================================================== */

function plannedPage(id) {
  const m = meta(id);
  const x = src(id);
  const ex = m.example;
  const fakeArt = h("div", { class: "art-fallback" }, icon(ex.art));
  const example = h("div", { class: "activity" }, h("div", { class: "activity-label", text: "Playing" }),
    h("div", { class: "presence" }, h("div", { class: "art" }, fakeArt),
      h("div", { class: "presence-text" }, h("div", { class: "presence-app", text: ex.app }),
        h("div", { class: "presence-line", text: ex.details }), h("div", { class: "presence-line", text: ex.state }),
        h("div", { class: "presence-time", text: "48:02 elapsed" }))));

  const el = h("div", { class: "page" },
    h("header", { class: "app-hero" },
      h("div", { class: "app-hero-main" }, tile(id, { size: "xl", soon: true }),
        h("div", {}, h("h1", { text: x.name }), h("div", { class: "status-row" }, h("span", { class: "chip soon", text: "Coming soon" })))),
      h("label", { class: "big-toggle", dataset: { disabled: "true" } }, "Show on Discord", sw({ big: true, disabled: true, label: `Show ${x.name} on Discord` }))),
    h("div", { class: "home-grid" },
      h("div", {}, h("p", { class: "planned-lede" }, m.lede[0], h("em", { text: m.lede[1] }), m.lede[2]),
        h("p", { class: "section-lede", style: "margin-top:18px" }, m.how)),
      h("section", { class: "profile", "aria-label": "Example" }, h("div", { style: "padding:20px 20px 0", class: "setting-hint", text: "How it could look" }), example)),
    h("section", { class: "section planned" }, sectionHead("Planned ", "A preview of what you’ll be able to change. None of it is active yet.", "settings"),
      h("div", { style: "display:grid;gap:28px" }, Object.entries(m.planned).map(([group, items]) => h("div", {},
        h("p", { class: "setting-hint", style: "margin:0 0 10px 4px;font-weight:600", text: group }),
        h("div", { class: "group" }, items.map(([t, hint]) => switchSetting(t, hint, sw({ disabled: true, label: t })))))))));
  return { el, update() {} };
}

/* ======================================================================
   Settings
   ==================================================================== */

/** Installs the newest version; the app restarts by itself. */
function installUpdate(btn) {
  return async () => {
    btn.disabled = true;
    btn.textContent = "Downloading…";
    try { await invoke("install_update"); }
    catch (e) { toast(String(e), "bad"); btn.disabled = false; btn.textContent = "Install and restart"; }
  };
}

function updatesControl() {
  const text = h("span", { class: "setting-hint" });
  const check = h("button", { class: "btn", type: "button", text: "Check now" });
  const install = h("button", { class: "btn primary", type: "button", text: "Install and restart", hidden: true });
  install.addEventListener("click", installUpdate(install));
  const paint = () => {
    install.hidden = !snap.updateAvailable;
    text.textContent = snap.updateAvailable ? `Version ${snap.updateAvailable} is ready (you have ${snap.version}).` : `You’re on ${snap.version}.`;
  };
  check.addEventListener("click", async () => {
    check.disabled = true;
    check.textContent = "Checking…";
    try {
      const v = await invoke("check_for_update");
      if (!v) toast("You’re on the newest version");
    } catch (e) { toast(String(e), "bad"); }
    check.disabled = false;
    check.textContent = "Check now";
    paint();
  });
  document.addEventListener("state-painted", paint);
  paint();
  return h("div", { style: "display:flex;align-items:center;gap:10px;flex-wrap:wrap" }, text, check, install);
}

function settingsPage() {
  const autostart = sw({ label: "Start with Windows", onchange: (on, input) => invoke("set_autostart", { enabled: on }).catch(() => { input.checked = !on; }) });
  const openOnLogin = sw({ label: "Open the window at sign-in", onchange: (on) => invoke("update_settings", { patch: { openWindowOnLogin: on } }).catch((e) => toast(String(e), "bad")) });
  const theme = segmented([["dark", "Dark"], ["light", "Light"], ["system", "System"]], store.get("theme") || "dark", (v) => {
    document.documentElement.dataset.theme = v;
    store.set("theme", v);
  }, { label: "Theme" });
  const isMac = /Mac/i.test(navigator.platform);

  const el = h("div", { class: "page", style: "max-width:860px" },
    h("header", { class: "hero" }, h("div", {}, h("h1", {}, "Settings for ", h("em", { text: "everything" }), "."), h("p", { class: "lede", text: "These apply to every app. Each app’s own settings live on its page." }))),
    h("section", {}, sectionHead("Starting ", null, "up"),
      h("div", { class: "group" },
        switchSetting(isMac ? "Open at login" : "Start with Windows", "Runs quietly in the tray when you sign in.", autostart),
        switchSetting("Open the window at sign-in", "Off keeps it tucked in the tray until you need it.", openOnLogin))),
    h("section", { class: "section" }, sectionHead("Look and ", null, "feel"),
      h("div", { class: "group" }, setting("Theme", "Dark suits the apps it sits next to. System follows Windows.", theme))),
    h("section", { class: "section" }, sectionHead("Several apps at ", null, "once"),
      h("div", { class: "group" }, setting("Which one shows",
        "The app you have in front. Switch from Roblox Studio to Blender and your status follows. Click away to Discord or a browser and it keeps the last one.",
        h("span", { class: "badge ok", text: "Automatic" })))),
    h("section", { class: "section" }, sectionHead("Keeping ", null, "up to date"),
      h("div", { class: "group" }, setting("Updates", "UltimatePresence checks for new versions by itself and installs them with one click.", updatesControl()))),
    h("section", { class: "section" }, sectionHead("Your ", null, "data"),
      h("div", { class: "group" }, setting("Settings folder", "Everything stays on this PC. Nothing is sent anywhere except to Discord.",
        h("button", { class: "btn", type: "button", onclick: () => invoke("open_external", { target: "settings-folder" }) }, "Open folder")))),
    h("section", { class: "section" },
      h("div", { class: "group" }, h("div", { class: "about" }, h("img", { src: "icon.svg", width: "84", height: "84", alt: "" }),
        h("div", {}, h("strong", { text: `UltimatePresence ${snap.version}` }),
          h("p", {}, "Built on ", h("button", { class: "link", type: "button", onclick: () => invoke("open_external", { target: "original-project" }) }, "StudioPresence by iArxic"),
            ". ISC licence. Instrument Serif and Figtree under the SIL Open Font Licence."))))));

  return { el, update(s) { autostart.checked = s.autostart; openOnLogin.checked = s.openWindowOnLogin; } };
}

/* ======================================================================
   Activity log
   ==================================================================== */

function logLine(e) {
  return h("li", { dataset: { level: e.level } },
    h("time", { text: timeFmtSec.format(new Date(e.timeMs)) }),
    h("span", { class: "lvl", "aria-label": e.level }),
    h("span", { class: "msg", text: e.message }));
}

function logPage() {
  let filter = store.get("logFilter") || "all";
  const list = h("ol", { class: "log", "aria-live": "polite" });
  const visible = (e) => filter === "all" || e.level !== "info";
  const render = () => {
    const items = logEntries.filter(visible).map(logLine);
    list.replaceChildren(...(items.length ? items : [h("li", { class: "log-empty", text: filter === "all" ? "No events yet." : "No problems. Everything’s running smoothly." })]));
    list.scrollTop = list.scrollHeight;
  };
  const filters = segmented([["all", "All"], ["problems", "Problems"]], filter, (v) => { filter = v; store.set("logFilter", v); render(); }, { label: "Filter" });
  const copy = h("button", { class: "btn", type: "button", text: "Copy" });
  copy.addEventListener("click", async () => {
    const text = logEntries.filter(visible).map((e) => `${timeFmtSec.format(new Date(e.timeMs))}  ${e.level.padEnd(5)}  ${e.message}`).join("\n");
    try { await navigator.clipboard.writeText(text); } catch {
      const ta = h("textarea"); ta.value = text; document.body.append(ta); ta.select(); document.execCommand("copy"); ta.remove();
    }
    toast("Log copied");
  });
  const clear = h("button", { class: "btn ghost", type: "button", text: "Clear" });
  clear.addEventListener("click", () => { invoke("clear_log"); logEntries.length = 0; render(); });

  const el = h("div", { class: "page log-page" },
    h("header", { class: "hero", style: "margin-bottom:28px" },
      h("div", {}, h("h1", {}, "What’s been ", h("em", { text: "happening" }), "."), h("p", { class: "lede", text: "Connections, plugin events and problems since the app started." })),
      h("div", { class: "log-tools" }, filters, copy, clear)),
    list);
  render();
  return {
    el,
    update() {},
    append(e) {
      if (!visible(e)) return;
      const near = list.scrollHeight - list.scrollTop - list.clientHeight < 40;
      list.querySelector(".log-empty")?.remove();
      list.append(logLine(e));
      if (near) list.scrollTop = list.scrollHeight;
    },
  };
}

/* ======================================================================
   Boot
   ==================================================================== */

function onLog(e) {
  logEntries.push(e);
  if (logEntries.length > 300) logEntries.shift();
  if (route === "log" && page && page.append) page.append(e);
  else if (e.level !== "info") unseenProblems++;
  if (snap) updateNav();
}

async function init() {
  document.querySelectorAll(".sidebar [data-route]").forEach((b) => b.addEventListener("click", () => go(b.dataset.route)));
  document.getElementById("me-pause").addEventListener("click", () => invoke("set_paused", { paused: !snap.paused }));

  await listen("state", (e) => {
    snap = e.payload;
    updateNav();
    if (page) page.update(snap);
    document.dispatchEvent(new Event("state-painted"));
  });
  await listen("log", (e) => onLog(e.payload));

  snap = await invoke("get_state");
  for (const e of await invoke("get_log")) logEntries.push(e);
  buildNav();
  refreshIntegrations();
  window.addEventListener("focus", refreshIntegrations);
  await document.fonts.ready.catch(() => {});
  go(route);
  invoke("ui_ready");
}

init();
