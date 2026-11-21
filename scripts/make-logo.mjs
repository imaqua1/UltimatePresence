// Draws the logo (italic serif "U" with the orange presence dot) into design/.
// The letter comes from the real Instrument Serif outline, converted to a path,
// because the icon renderer has no fonts.
// Usage: npm run logo   (then `npm run icons` to rebuild the app icons)
import opentype from "opentype.js";
import { writeFileSync } from "node:fs";

const ITALIC = opentype.loadSync("design/fonts/InstrumentSerif-Italic.ttf");

/** Path for `ch` scaled so its ink box is `h` tall, centred on (cx, cy). */
function glyph(font, ch, h, cx, cy) {
  const probe = font.getPath(ch, 0, 0, 1000).getBoundingBox();
  const size = 1000 * (h / (probe.y2 - probe.y1));
  const bb = font.getPath(ch, 0, 0, size).getBoundingBox();
  return font.getPath(ch, cx - (bb.x1 + bb.x2) / 2, cy - (bb.y1 + bb.y2) / 2, size).toPathData(2);
}

const BG = `<linearGradient id="bg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#1A1714"/><stop offset="1" stop-color="#0B0A09"/></linearGradient>`;
const OR = `<linearGradient id="or" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#FF9A4D"/><stop offset="1" stop-color="#F0590E"/></linearGradient>`;

/**
 * full: the tray version fills the whole tile and uses a heavier letter so it survives 16 px.
 * idle: the greyed tray icon shown while paused.
 */
function logo(full, idle) {
  const tile = full
    ? `<rect width="1024" height="1024" rx="232" fill="url(#bg)"/>`
    : `<rect x="64" y="64" width="896" height="896" rx="216" fill="url(#bg)"/>`;
  const color = idle ? "#A8A29E" : "url(#or)";
  const d = glyph(ITALIC, "U", full ? 660 : 560, full ? 488 : 492, 532);
  // The letter is stroked in its own colour to thicken it for small sizes.
  const letter = `<path d="${d}" fill="${color}" stroke="${color}" stroke-width="${full ? 44 : 22}" stroke-linejoin="round"/>`;
  const dot = `<circle cx="${full ? 846 : 800}" cy="${full ? 206 : 240}" r="${full ? 92 : 62}" fill="${idle ? "#D6D3D1" : "#FFB27A"}"/>`;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024"><defs>${BG}${OR}</defs>${tile}${letter}${dot}</svg>\n`;
}

writeFileSync("design/icon.svg", logo(false, false));
writeFileSync("design/tray.svg", logo(true, false));
writeFileSync("design/tray-idle.svg", logo(true, true));
console.log("design/icon.svg, tray.svg, tray-idle.svg written");
