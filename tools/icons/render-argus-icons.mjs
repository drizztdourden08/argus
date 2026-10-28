/* @layer tooling-scripts @kind logic */
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Resvg } from '@resvg/resvg-js';

const root = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const source = join(root, 'apps', 'web', 'assets', 'argus-logo.svg');
const outDir = join(root, 'crates', 'argus-native', 'assets');

const SIZES = [16, 24, 32, 48, 256];
const INKS = { white: '#ffffff', black: '#000000' };
const PRIMARY = '#8a8a96';
const SQUARE = { x: 52, y: 62.5, side: 1150 };
const BOLD = { 16: 20, 24: 14, 32: 8 };
const ICO_HEAD = 6;
const ICO_ENTRY = 16;
const ICO_LARGEST = 256;

const tinted = (svg, color, size) => svg
  .replace('viewBox="0 0 1254 1254"', `viewBox="${SQUARE.x} ${SQUARE.y} ${SQUARE.side} ${SQUARE.side}"`)
  .replace(
    'fill="currentColor"',
    `fill="${color}" stroke="${color}" stroke-width="${BOLD[size] ?? 0}" stroke-linejoin="round"`,
  );

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
  const images = SIZES.map((size) => ({ size, data: png(tinted(svg, PRIMARY, size), size) }));
  writeFileSync(join(outDir, 'argus.ico'), ico(images));
};

mkdirSync(outDir, { recursive: true });
const svg = readFileSync(source, 'utf8');
writeInks(svg);
writeIdentity(svg);
