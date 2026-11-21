// Draws the large "state" icons in the same style as StudioPresence's script icons:
// a flat near-black square, one chunky glyph in a single bright colour, soft edges.
// Usage: npm run discord-images  → dist/discord-assets/icon_*.png
import { execSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const BG = "#111111";
// The originals have slightly soft, glowing edges.
const soft = `<filter id="soft" x="-10%" y="-10%" width="120%" height="120%"><feGaussianBlur stdDeviation="2.4"/></filter>`;

const rect = (x, y, w, h, r, fill) => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${r}" fill="${fill}"/>`;
const diamond = (x, y, r, fill, rx = 0.18) =>
  `<rect x="${x - r * 0.707}" y="${y - r * 0.707}" width="${r * 1.414}" height="${r * 1.414}" rx="${r * rx}" transform="rotate(45 ${x} ${y})" fill="${fill}"/>`;
const sparkle = (x, y, r, fill) => {
  const k = r * 0.2;
  return `<path d="M${x} ${y - r} C${x + k} ${y - k} ${x + k} ${y - k} ${x + r} ${y} C${x + k} ${y + k} ${x + k} ${y + k} ${x} ${y + r} C${x - k} ${y + k} ${x - k} ${y + k} ${x - r} ${y} C${x - k} ${y - k} ${x - k} ${y - k} ${x} ${y - r}Z" fill="${fill}"/>`;
};

const YELLOW = "#FFC43D";
const GREEN = "#2BE36A";
const PURPLE = "#A78BFA";
const PINK = "#FF6AD5";

const icons = {
  // Brick wall: building with parts
  icon_building: [
    rect(150, 150, 140, 80, 14, YELLOW), rect(310, 150, 140, 80, 14, YELLOW),
    rect(150, 260, 60, 80, 14, YELLOW), rect(230, 260, 140, 80, 14, YELLOW), rect(390, 260, 60, 80, 14, YELLOW),
    rect(150, 370, 140, 80, 14, YELLOW), rect(310, 370, 140, 80, 14, YELLOW),
  ],
  // Play button
  icon_playtest: [
    `<path d="M215 160 Q215 140 233 150 L440 280 Q456 300 440 320 L233 450 Q215 460 215 440 Z" fill="${GREEN}"/>`,
  ],
  // Keyframes on a timeline
  icon_animating: [
    rect(110, 286, 380, 28, 14, PURPLE),
    diamond(150, 300, 52, PURPLE),
    diamond(450, 300, 52, PURPLE),
    diamond(300, 300, 168, BG),
    diamond(300, 300, 148, PURPLE),
  ],
  // Particles
  icon_vfx: [
    sparkle(275, 330, 165, PINK),
    sparkle(430, 175, 70, PINK),
    rect(150, 150, 46, 46, 12, PINK),
  ],
  // Keyframe + particle
  icon_combo: [
    diamond(250, 340, 130, PURPLE),
    `<g>${sparkle(400, 205, 120, BG)}${sparkle(400, 205, 98, PINK)}</g>`,
  ],
};

mkdirSync("dist/discord-assets", { recursive: true });
for (const [name, parts] of Object.entries(icons)) {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 600"><defs>${soft}</defs><rect width="600" height="600" fill="${BG}"/><g filter="url(#soft)">${parts.join("")}</g></svg>\n`;
  const out = mkdtempSync(join(tmpdir(), "up-icon-"));
  const svgPath = join(out, "${name}.svg");
  writeFileSync(svgPath, svg);
  execSync(`npx tauri icon "${svgPath}" -o "${out}" --png 512`, { stdio: "ignore" });
  copyFileSync(join(out, "512x512.png"), `dist/discord-assets/${name}.png`);
  rmSync(out, { recursive: true, force: true });
  console.log(`dist/discord-assets/${name}.png`);
}
