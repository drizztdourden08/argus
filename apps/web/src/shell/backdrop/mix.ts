/* @layer renderer-app @kind logic */
import { clampChannel } from './clamp-channel';
import type { Rgb } from './color-math.type';

const mix = (from: Rgb, to: Rgb, t: number): Rgb => [
  clampChannel(from[0] + (to[0] - from[0]) * t),
  clampChannel(from[1] + (to[1] - from[1]) * t),
  clampChannel(from[2] + (to[2] - from[2]) * t),
];

export { mix };
