/* @layer tooling-scripts @kind logic */
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Resvg } from '@resvg/resvg-js';

const root = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const source = join(root, 'apps', 'web', 'assets', 'argus-logo.svg');
const outDir = join(root, 'crates', 'argus-native', 'assets');

const TRAY_SIZES = [16, 20, 24, 28, 32, 40, 48, 64];
const ICO_SIZES = [16, 20, 24, 32, 40, 48, 256];
const SIZES = [...new Set([...TRAY_SIZES, ...ICO_SIZES])];
const INKS = { white: '#ffffff', black: '#000000' };
const PRIMARY = '#8a8a96';
const MARK = { x: 67, y: 270, width: 1120, height: 684 };
const CRISP_UP_TO = 16;
const STROKE_PX = { 16: 0.3, 20: 0.3, 24: 0.3, 28: 0.3, 32: 0.3 };
const ICO_HEAD = 6;
const ICO_ENTRY = 16;
const ICO_LARGEST = 256;

const margin = (size) => Math.round(size / 32);

/**
 * The view box that puts the painted mark (outline plus whole-pixel stroke) inside the square
 * with the same margin on every side, its top edge on a pixel boundary, so a horizontal edge
 * of the mark falls between two rows of pixels instead of across one.
 */
const frame = (size) => {
  const stroke = STROKE_PX[size] ?? 0;
  const scale = (size - 2 * margin(size) - stroke) / MARK.width;
  const unit = 1 / scale;
  const painted = MARK.height * scale + stroke;
  const top = Math.round((size - painted) / 2);
  const x = MARK.x - (stroke * unit) / 2 - margin(size) * unit;
  const y = MARK.y - (stroke * unit) / 2 - top * unit;
  return { x, y, side: size * unit, stroke: stroke * unit };
};

const tinted = (svg, color, size) => {
  const { x, y, side, stroke } = frame(size);
  const crisp = size <= CRISP_UP_TO ? ' shape-rendering="crispEdges"' : '';
  const ink = stroke > 0
    ? `fill="${color}" stroke="${color}" stroke-width="${stroke}" stroke-linejoin="miter"`
    : `fill="${color}"`;
  return svg
    .replace('viewBox="0 0 1254 1254"', `viewBox="${x} ${y} ${side} ${side}"${crisp}`)
    .replace('fill="currentColor"', ink);
};

const png = (svg, size) => new Resvg(svg, { fitTo: { mode: 'width', value: size } }).render().asPng();

const writeEntry = (head, index, offset, { size, data }) => {
  const at = ICO_HEAD + ICO_ENTRY * index;
  const side = size === ICO_LARGEST ? 0 : size;
  head.writeUInt8(side, at);
  head.writeUInt8(side, at + 1);
  head.writeUInt16LE(1, at + 4);
  head.writeUInt16LE(32, at + 6);
  head.writeUInt32LE(data.length, at + 8);
  head.writeUInt32LE(offset, at + 12);
};

const ico = (images) => {
  const head = Buffer.alloc(ICO_HEAD + ICO_ENTRY * images.length);
  head.writeUInt16LE(1, 2);
  head.writeUInt16LE(images.length, 4);
  let offset = head.length;
  images.forEach((image, index) => {
    writeEntry(head, index, offset, image);
    offset += image.data.length;
  });
  return Buffer.concat([head, ...images.map(({ data }) => data)]);
};

const writeInks = (svg) => {
  for (const [name, color] of Object.entries(INKS)) {
    for (const size of SIZES) {
      writeFileSync(join(outDir, `argus-${name}-${size}.png`), png(tinted(svg, color, size), size));
    }
  }
};

const writeIdentity = (svg) => {
  const images = ICO_SIZES.map((size) => ({ size, data: png(tinted(svg, PRIMARY, size), size) }));
  writeFileSync(join(outDir, 'argus.ico'), ico(images));
};

mkdirSync(outDir, { recursive: true });
const svg = readFileSync(source, 'utf8');
writeInks(svg);
writeIdentity(svg);
