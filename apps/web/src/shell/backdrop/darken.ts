/* @layer renderer-app @kind logic */
import { clampChannel } from './clamp-channel';
import type { Rgb } from './color-math.type';

const darken = (color: Rgb, amount: number): Rgb => [
  clampChannel(color[0] * (1 - amount)),
  clampChannel(color[1] * (1 - amount)),
  clampChannel(color[2] * (1 - amount)),
];

export { darken };
