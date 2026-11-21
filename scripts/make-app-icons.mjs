// Regenerates the app and tray icons in src-tauri/icons from the logo in design/.
// Usage: npm run icons
import { execSync } from "node:child_process";
import { copyFileSync, mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const run = (cmd) => execSync(cmd, { stdio: "inherit" });
const icons = "src-tauri/icons";

run(`npx tauri icon design/icon.svg -o ${icons}`);
for (const name of ["tray", "tray-idle"]) {
  const out = mkdtempSync(join(tmpdir(), "up-icon-"));
  run(`npx tauri icon design/${name}.svg -o "${out}"`);
  copyFileSync(join(out, "32x32.png"), join(icons, `${name}.png`));
  rmSync(out, { recursive: true, force: true });
}

// Keep only what the app uses: the bundle icons in tauri.conf.json, the tray icons,
// and icon.png (window icon on macOS/Linux). The CLI also makes mobile/Store sizes.
const keep = new Set(["32x32.png", "128x128.png", "128x128@2x.png", "icon.icns", "icon.ico", "icon.png", "tray.png", "tray-idle.png"]);
for (const entry of readdirSync(icons)) if (!keep.has(entry)) rmSync(join(icons, entry), { recursive: true, force: true });
