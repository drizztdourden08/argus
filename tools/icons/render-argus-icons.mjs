/* @layer tooling-scripts @kind logic */
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Resvg } from '@resvg/resvg-js';
import { logoSvg, writeLogos } from './generate-argus-logos.mjs';

const root = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const outDir = join(root, 'crates', 'argus-native', 'assets');

const TRAY_SIZES = [16, 20, 24, 28, 32, 40, 48, 64];
const ICO_SIZES = [16, 20, 24, 32, 40, 48, 256];
const SIZES = [...new Set([...TRAY_SIZES, ...ICO_SIZES, 256])];
const THEMES = ['white', 'black'];
const SMALL_UP_TO = 28;
const MEDIUM_UP_TO = 48;
const ICO_HEAD = 6;
const ICO_ENTRY = 16;
const ICO_LARGEST = 256;

const levelOf = (size) => {
  if (size <= SMALL_UP_TO) return 'small';
  return size <= MEDIUM_UP_TO ? 'medium' : 'full';
};

const png = (size) => new Resvg(logoSvg('a', levelOf(size), size), { fitTo: { mode: 'width', value: size } }).render().asPng();
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

const writeInks = () => {
  for (const size of SIZES) {
    const data = png(size);
    for (const name of THEMES) writeFileSync(join(outDir, `argus-${name}-${size}.png`), data);
  }
};

const writeIdentity = () => {
  const images = ICO_SIZES.map((size) => ({ size, data: png(size) }));
  writeFileSync(join(outDir, 'argus.ico'), ico(images));
};

mkdirSync(outDir, { recursive: true });
writeLogos();
writeInks();
writeIdentity();
