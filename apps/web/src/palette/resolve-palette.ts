/* @layer renderer-app @kind logic */
import { legacyKeys } from '../data/boards/legacy-keys';
import type { BoardInfo } from '../data/boards/boards.type';
import { BASIC, KEY_PALETTES, PALETTES } from './palette.constants';
import type { Palette } from './palette.type';

const isPalette = (value: unknown): value is Palette => PALETTES.some((palette) => palette === value);

const keyPalette = (key: string, boards?: readonly BoardInfo[]): Palette | undefined => {
  const found = legacyKeys(key, boards).find((candidate) => Object.hasOwn(KEY_PALETTES, candidate));
  return found === undefined ? undefined : KEY_PALETTES[found];
};

const resolvePalette = (key: string, setting?: unknown, boards?: readonly BoardInfo[]): Palette | undefined => {
  if (setting === BASIC) return undefined;
  if (isPalette(setting)) return setting;
  return keyPalette(key, boards);
};

export { resolvePalette };
