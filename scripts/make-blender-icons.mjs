// Draws the Blender images in the same flat style as the Roblox Studio ones (near-black
// square, one bright glyph, soft edges) and renders 512×512 PNGs for upload to Discord
// (Rich Presence → Art Assets). Also draws the shared Zzz idle icon.
//
// Eight category pictures (related categories share one), plus the Blender badge, whose
// shape is traced from the official Blender logo (single colour).
// Usage: npm run discord-images
//   → dist/discord-assets/blender/*.png, dist/discord-assets/icon_idle.png
import { execSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const BG = "#111111";
const soft = `<filter id="soft" x="-10%" y="-10%" width="120%" height="120%"><feGaussianBlur stdDeviation="2.4"/></filter>`;
const C = {
  orange: "#FF8A2B", peach: "#FFB27A", cyan: "#3DD6E0", purple: "#B57BFF",
  blue: "#5AA8FF", yellow: "#FFC43D", red: "#FF5A5A", teal: "#2DD4BF", sky: "#8AB4FF",
};

const rect = (x, y, w, h, r, fill, extra = "") => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${r}" fill="${fill}" ${extra}/>`;
const circle = (x, y, r, fill, extra = "") => `<circle cx="${x}" cy="${y}" r="${r}" fill="${fill}" ${extra}/>`;
const ellipse = (x, y, rx, ry, fill, extra = "") => `<ellipse cx="${x}" cy="${y}" rx="${rx}" ry="${ry}" fill="${fill}" ${extra}/>`;
const line = (x1, y1, x2, y2, w, color, extra = "") =>
  `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${color}" stroke-width="${w}" stroke-linecap="round" ${extra}/>`;
const path = (d, fill, extra = "") => `<path d="${d}" fill="${fill}" ${extra}/>`;
const stroke = (d, w, color, extra = "") =>
  `<path d="${d}" fill="none" stroke="${color}" stroke-width="${w}" stroke-linecap="round" stroke-linejoin="round" ${extra}/>`;

// Official Blender logo geometry (viewBox 0 0 181 148): outer shape with the eye cut out,
// and the core. Single colour: orange shape, dark ring, orange core.
const LOGO_OUTER = "m231-330c0.013 0.74 0.249 2.18 0.603 3.3 0.744 2.38 2.01 4.58 3.76 6.51 1.8 1.99 4.02 3.59 6.58 4.73 2.69 1.19 5.61 1.8 8.64 1.8 3.03-4e-3 5.95-0.624 8.64-1.83 2.56-1.15 4.78-2.75 6.58-4.75 1.76-1.95 3.02-4.15 3.76-6.530.375-1.2 0.612-2.42 0.707-3.64 0.093-1.2 0.054-2.41-0.117-3.62-0.334-2.35-1.15-4.56-2.4-6.56-1.14-1.85-2.62-3.46-4.38-4.82l4e-3 -3e-3 -17.7-13.6c-0.016-0.012-0.029-0.025-0.046-0.036-1.16-0.892-3.12-0.889-4.39 5e-3 -1.29 0.904-1.44 2.4-0.29 3.34l-5e-3 5e-3 7.39 6.01-22.5 0.024h-0.03c-1.86 2e-3 -3.65 1.22-4 2.77-0.364 1.57 0.9 2.88 2.84 2.88l-3e-3 7e-3 11.4-0.022-20.4 15.6c-0.026 0.019-0.054 0.039-0.078 0.058-1.92 1.47-2.54 3.92-1.33 5.46 1.23 1.57 3.84 1.58 5.78 9e-3l11.1-9.1s-0.162 1.23-0.149 1.96zm28.6 4.11c-2.29 2.33-5.5 3.66-8.96 3.66-3.47 6e-3 -6.68-1.3-8.97-3.63-1.12-1.14-1.94-2.44-2.45-3.83-0.497-1.37-0.69-2.82-0.562-4.28 0.121-1.43 0.547-2.8 1.23-4.03 0.668-1.21 1.59-2.31 2.72-3.24 2.23-1.81 5.06-2.8 8.02-2.8 2.97-4e-3 5.8 0.969 8.03 2.78 1.13 0.924 2.05 2.02 2.72 3.23 0.683 1.23 1.11 2.59 1.23 4.03 0.126 1.46-0.067 2.91-0.564 4.28-0.508 1.4-1.33 2.7-2.45 3.84z";
const LOGO_CORE = "m243-334c0.106-1.89 1.03-3.56 2.43-4.74 1.37-1.16 3.21-1.87 5.23-1.87 2.01 0 3.85 0.709 5.22 1.87 1.4 1.18 2.32 2.85 2.43 4.74 0.106 1.94-0.675 3.75-2.04 5.09-1.4 1.36-3.38 2.22-5.61 2.22s-4.22-0.854-5.61-2.22c-1.37-1.34-2.15-3.14-2.04-5.08z";
const blenderLogo = (fill) =>
  `<g transform="translate(78 122) scale(2.45)"><g transform="matrix(.281 0 0 .281 -41.8 -43.7)">` +
  `<g transform="matrix(11.1 0 0 11.1 -2215 4153)"><path d="${LOGO_OUTER}" fill="${fill}" fill-rule="evenodd"/><path d="${LOGO_CORE}" fill="${fill}"/></g>` +
  `</g></g>`;

const icons = {
  // Badge (and the picture for scripting / video editing)
  bl_blender: [blenderLogo(C.orange)],

  // Wireframe cube with its corner points: the universal 3D-modelling symbol
  bl_modeling: (() => {
    const v = { t: [300, 118], ul: [142, 206], ur: [458, 206], c: [300, 294], ll: [142, 392], lr: [458, 392], b: [300, 480] };
    const edges = [["t", "ul"], ["t", "ur"], ["ul", "c"], ["ur", "c"], ["c", "b"], ["ul", "ll"], ["ur", "lr"], ["ll", "b"], ["lr", "b"]];
    return [
      ...edges.map(([a, b]) => line(...v[a], ...v[b], 22, C.orange)),
      ...Object.values(v).map(([x, y]) => circle(x, y, 30, C.orange)),
      ...Object.values(v).map(([x, y]) => circle(x, y, 13, BG)),
    ];
  })(),

  // A sculpted bust: head, neck and shoulders on a plinth
  bl_sculpting: [
    ellipse(300, 205, 92, 112, C.peach),
    rect(262, 285, 76, 90, 20, C.peach),
    path("M110 478 C 118 410, 190 372, 262 362 L 338 362 C 410 372, 482 410, 490 478 Z", C.peach),
    rect(150, 492, 300, 30, 12, C.peach),
  ],

  // UV checker: the test texture everyone recognises
  bl_uv: (() => {
    const out = [rect(120, 120, 360, 360, 30, C.cyan)];
    for (let r = 0; r < 4; r++) for (let c = 0; c < 4; c++) if ((r + c) % 2 === 1) out.push(rect(120 + c * 90, 120 + r * 90, 90, 90, 0, BG));
    return [`<clipPath id="uvclip"><rect x="120" y="120" width="360" height="360" rx="30"/></clipPath>`, `<g clip-path="url(#uvclip)">${out.join("")}</g>`];
  })(),

  // Paint brush with a drop of paint (texture painting, Grease Pencil drawing)
  bl_painting: [
    line(470, 130, 345, 255, 46, C.purple),
    `<line x1="345" y1="255" x2="300" y2="300" stroke="${C.purple}" stroke-width="78"/>`,
    path("M322 278 C 300 360, 240 410, 150 432 C 170 350, 220 300, 322 278 Z", C.purple),
    path("M170 470 C 170 470, 128 520, 150 545 C 165 560, 190 548, 190 525 C 190 505, 170 470, 170 470 Z", C.purple),
  ],

  // Material preview ball with a highlight and its shadow
  bl_shading: [
    ellipse(300, 492, 150, 30, C.blue, `opacity="0.35"`),
    circle(300, 280, 175, C.blue),
    ellipse(232, 205, 52, 30, BG, `transform="rotate(-35 232 205)"`),
    circle(300, 182, 12, BG),
  ],

  // The bouncing ball with its motion trail (animating, rigging)
  bl_animating: [
    stroke("M110 120 Q 200 140 270 452 Q 330 230 470 200", 14, C.yellow, `opacity="0.45" stroke-dasharray="2 30"`),
    line(80, 500, 520, 500, 18, C.yellow, `opacity="0.6"`),
    circle(130, 140, 40, C.yellow, `opacity="0.35"`),
    circle(222, 300, 46, C.yellow, `opacity="0.55"`),
    ellipse(272, 448, 66, 38, C.yellow, `opacity="0.8"`),
    circle(430, 230, 78, C.yellow),
  ],

  // A camera
  bl_rendering: [
    rect(150, 168, 120, 64, 18, C.red),
    rect(100, 206, 400, 262, 40, C.red),
    circle(300, 338, 98, BG),
    circle(300, 338, 64, C.red),
    circle(278, 316, 16, BG),
    circle(442, 248, 16, BG),
  ],

  // Node boxes wired together (compositing, geometry nodes)
  bl_nodes: [
    stroke("M246 214 C 300 214, 300 300, 356 300", 20, C.teal),
    stroke("M246 386 C 300 386, 300 300, 356 300", 20, C.teal),
    rect(96, 150, 150, 128, 24, C.teal), rect(96, 182, 150, 10, 0, BG),
    rect(96, 322, 150, 128, 24, C.teal), rect(96, 354, 150, 10, 0, BG),
    rect(356, 228, 150, 144, 24, C.teal), rect(356, 260, 150, 10, 0, BG),
    circle(246, 214, 18, C.teal), circle(246, 386, 18, C.teal), circle(356, 300, 18, C.teal),
  ],
};

// Shared with Roblox Studio: Zzz for idle.
const idle = [
  stroke("M130 330 L290 330 L130 490 L290 490", 46, C.sky),
  stroke("M320 220 L430 220 L320 330 L430 330", 38, C.sky),
  stroke("M430 120 L500 120 L430 190 L500 190", 30, C.sky),
];

function render(name, parts, outDir) {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 600"><defs>${soft}</defs><rect width="600" height="600" fill="${BG}"/><g filter="url(#soft)">${parts.join("")}</g></svg>\n`;
  const out = mkdtempSync(join(tmpdir(), "up-icon-"));
  const svgPath = join(out, "${name}.svg");
  writeFileSync(svgPath, svg);
  execSync(`npx tauri icon "${svgPath}" -o "${out}" --png 512`, { stdio: "ignore" });
  copyFileSync(join(out, "512x512.png"), join(outDir, `${name}.png`));
  rmSync(out, { recursive: true, force: true });
  console.log(join(outDir, `${name}.png`));
}

rmSync("dist/discord-assets/blender", { recursive: true, force: true });
mkdirSync("dist/discord-assets/blender", { recursive: true });
for (const [name, parts] of Object.entries(icons)) render(name, parts, "dist/discord-assets/blender");
render("icon_idle", idle, "dist/discord-assets");
copyFileSync("dist/discord-assets/icon_idle.png", "dist/discord-assets/blender/icon_idle.png");
