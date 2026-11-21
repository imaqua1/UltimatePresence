// Regenerates all app + tray icons from the SVGs in assets/ using the Tauri CLI.
// Usage: npm run icons
import { execSync } from "node:child_process";
import { copyFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const run = (cmd) => execSync(cmd, { stdio: "inherit" });
const icons = "src-tauri/icons";

run(`npx tauri icon assets/icon.svg -o ${icons}`);
for (const name of ["tray", "tray-idle"]) {
  const out = mkdtempSync(join(tmpdir(), "up-icon-"));
  run(`npx tauri icon assets/${name}.svg -o "${out}"`);
  copyFileSync(join(out, "32x32.png"), join(icons, `${name}.png`));
  rmSync(out, { recursive: true, force: true });
}
// Mobile/Store icons aren't used by this desktop app.
for (const extra of ["android", "ios"]) rmSync(join(icons, extra), { recursive: true, force: true });
