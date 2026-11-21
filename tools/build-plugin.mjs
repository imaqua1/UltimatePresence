// Builds plugin/ into dist/UltimatePresence.rbxmx (a Roblox model file Studio loads as a
// local plugin). Uses Rojo's naming rules, so no Rojo install is needed:
//   name.server.luau -> Script, name.client.luau -> LocalScript, name.luau -> ModuleScript,
//   folders -> Folder. The root folder is named "StudioPresence" because the plugin's
//   modules find each other with script:FindFirstAncestor("StudioPresence").
// Usage: node tools/build-plugin.mjs [--install]
//   --install copies it into %LOCALAPPDATA%\Roblox\Plugins and disables the original
//   studiopresence.rbxm there (renamed to .disabled, so it can be restored).
import { readdirSync, readFileSync, statSync, writeFileSync, mkdirSync, existsSync, renameSync, copyFileSync } from "node:fs";
import { join } from "node:path";

let ref = 0;
const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
const cdata = (s) => `<![CDATA[${s.replace(/]]>/g, "]]]]><![CDATA[>")}]]>`;

function item(className, name, source) {
  const props = [`<string name="Name">${esc(name)}</string>`];
  if (source != null) props.push(`<ProtectedString name="Source">${cdata(source)}</ProtectedString>`);
  return { open: `<Item class="${className}" referent="RBX${ref++}"><Properties>${props.join("")}</Properties>`, close: "</Item>" };
}

function walk(dir) {
  const out = [];
  for (const entry of readdirSync(dir).sort()) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) {
      const it = item("Folder", entry);
      out.push(it.open, ...walk(full), it.close);
      continue;
    }
    const m = /^(.*?)(\.server|\.client)?\.luau?$/.exec(entry);
    if (!m) continue;
    const cls = m[2] === ".server" ? "Script" : m[2] === ".client" ? "LocalScript" : "ModuleScript";
    const it = item(cls, m[1], readFileSync(full, "utf8").replace(/\r\n/g, "\n"));
    out.push(it.open, it.close);
  }
  return out;
}

const root = item("Folder", "StudioPresence");
const xml = `<roblox xmlns:xmime="http://www.w3.org/2005/05/xmlmime" version="4">${root.open}${walk("plugin").join("")}${root.close}</roblox>\n`;
mkdirSync("dist", { recursive: true });
writeFileSync("dist/UltimatePresence.rbxmx", xml);
console.log(`dist/UltimatePresence.rbxmx (${ref} instances, ${xml.length} bytes)`);

if (process.argv.includes("--install")) {
  const plugins = join(process.env.LOCALAPPDATA, "Roblox", "Plugins");
  mkdirSync(plugins, { recursive: true });
  const old = join(plugins, "studiopresence.rbxm");
  if (existsSync(old)) {
    renameSync(old, `${old}.disabled`);
    console.log(`disabled the original: ${old} -> .disabled`);
  }
  copyFileSync("dist/UltimatePresence.rbxmx", join(plugins, "UltimatePresence.rbxmx"));
  console.log(`installed to ${plugins}`);
}
