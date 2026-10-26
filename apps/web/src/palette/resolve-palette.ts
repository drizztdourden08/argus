/* @layer renderer-app @kind logic */
import { BASIC, KEY_PALETTES, PALETTES } from './palette.constants';
import type { Palette } from './palette.type';

const isPalette = (value: unknown): value is Palette => PALETTES.some((palette) => palette === value);

const resolvePalette = (key: string, setting?: unknown): Palette | undefined => {
  if (setting === BASIC) return undefined;
  if (isPalette(setting)) return setting;
  return Object.hasOwn(KEY_PALETTES, key) ? KEY_PALETTES[key] : undefined;
};

export { resolvePalette };
