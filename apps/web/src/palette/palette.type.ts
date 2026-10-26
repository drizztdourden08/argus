/* @layer renderer-app @kind types */
import type { PALETTES } from './palette.constants';

type Palette = (typeof PALETTES)[number];

export type { Palette };
