/* @layer renderer-app @kind logic */
import type { Rgb } from './color-math.type';

const toHex = (color: Rgb): string => `#${color.map((channel) => channel.toString(16).padStart(2, '0')).join('')}`;

export { toHex };
