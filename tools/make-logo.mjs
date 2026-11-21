// Builds the logo SVGs from the real Instrument Serif glyph outlines (fonts aren't
// available to the icon renderer, so the letter is converted to a path).
// Usage: node tools/make-logo.mjs            -> writes assets/logo-variants/*.svg
//        node tools/make-logo.mjs <variant>  -> also writes assets/icon.svg + tray SVGs
import opentype from "opentype.js";
import { mkdirSync, writeFileSync } from "node:fs";

const ROMAN = opentype.loadSync("tools/fonts-src/InstrumentSerif-Regular.ttf");
const ITALIC = opentype.loadSync("tools/fonts-src/InstrumentSerif-Italic.ttf");

/** Path for `ch` scaled so its ink box is `h` tall, centred on (cx, cy). */
function glyph(font, ch, h, cx, cy) {
  const probe = font.getPath(ch, 0, 0, 1000).getBoundingBox();
  const scale = h / (probe.y2 - probe.y1);
  const size = 1000 * scale;
  const bb = font.getPath(ch, 0, 0, size).getBoundingBox();
  const x = cx - (bb.x1 + bb.x2) / 2;
  const y = cy - (bb.y1 + bb.y2) / 2;
  const p = font.getPath(ch, x, y, size);
  return { d: p.toPathData(2), box: p.getBoundingBox() };
}

const BG = `<linearGradient id="bg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#1A1714"/><stop offset="1" stop-color="#0B0A09"/></linearGradient>`;
const OR = `<linearGradient id="or" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#FF9A4D"/><stop offset="1" stop-color="#F0590E"/></linearGradient>`;
const tile = (full, fill = "url(#bg)") =>
  full ? `<rect width="1024" height="1024" rx="232" fill="${fill}"/>` : `<rect x="64" y="64" width="896" height="896" rx="216" fill="${fill}"/>`;

/** The letter, thickened by stroking it in its own colour so it survives 16–32 px. */
const letter = (g, color, full) =>
  `<path d="${g.d}" fill="${color}" stroke="${color}" stroke-width="${full ? 44 : 22}" stroke-linejoin="round"/>`;
const dot = (full, color) =>
  `<circle cx="${full ? 846 : 800}" cy="${full ? 206 : 240}" r="${full ? 92 : 62}" fill="${color}"/>`;

const variants = {
  // Italic serif U, orange, with the presence dot
  "italic-dot": (full, idle) => {
    const g = glyph(ITALIC, "U", full ? 660 : 560, full ? 488 : 492, 532);
    return tile(full) + letter(g, idle ? "#A8A29E" : "url(#or)", full) + dot(full, idle ? "#D6D3D1" : "#FFB27A");
  },
  // Roman serif U, orange, no dot
  roman: (full, idle) => {
    const g = glyph(ROMAN, "U", full ? 660 : 560, 512, 520);
    return tile(full) + letter(g, idle ? "#A8A29E" : "url(#or)", full);
  },
  // Italic U in warm white, orange dot
  "italic-light": (full, idle) => {
    const g = glyph(ITALIC, "U", full ? 660 : 560, full ? 488 : 492, 532);
    return tile(full) + letter(g, idle ? "#A8A29E" : "#F4EEE8", full) + dot(full, idle ? "#D6D3D1" : "#FF7A2A");
  },
  // Orange tile, black italic U
  inverse: (full, idle) => {
    const g = glyph(ITALIC, "U", full ? 680 : 580, 500, 528);
    return tile(full, idle ? "#78716C" : "url(#or)") + letter(g, "#0B0A09", full);
  },
};

const svg = (body) => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024"><defs>${BG}${OR}</defs>${body}</svg>\n`;

mkdirSync("assets/logo-variants", { recursive: true });
for (const [name, fn] of Object.entries(variants)) writeFileSync(`assets/logo-variants/${name}.svg`, svg(fn(false, false)));

const pick = process.argv[2];
if (pick) {
  const fn = variants[pick];
  if (!fn) throw new Error(`unknown variant ${pick}`);
  writeFileSync("assets/icon.svg", svg(fn(false, false)));
  writeFileSync("assets/tray.svg", svg(fn(true, false)));
  writeFileSync("assets/tray-idle.svg", svg(fn(true, true)));
  console.log(`assets/icon.svg + tray SVGs written from "${pick}"`);
}
