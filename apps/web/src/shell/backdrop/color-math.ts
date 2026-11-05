/* @layer renderer-app @kind logic */
type Rgb = readonly [number, number, number];

const clamp = (value: number): number => Math.min(255, Math.max(0, Math.round(value)));

const darken = (color: Rgb, amount: number): Rgb => [
  clamp(color[0] * (1 - amount)),
  clamp(color[1] * (1 - amount)),
  clamp(color[2] * (1 - amount)),
];

const mix = (from: Rgb, to: Rgb, t: number): Rgb => [
  clamp(from[0] + (to[0] - from[0]) * t),
  clamp(from[1] + (to[1] - from[1]) * t),
  clamp(from[2] + (to[2] - from[2]) * t),
];

const toHex = (color: Rgb): string => `#${color.map((channel) => channel.toString(16).padStart(2, '0')).join('')}`;

const toVec4 = (hex: string): [number, number, number, number] => {
  const value = Number.parseInt(hex.replace('#', ''), 16);
  return [((value >> 16) & 255) / 255, ((value >> 8) & 255) / 255, (value & 255) / 255, 1];
};

export { darken, mix, toHex, toVec4 };
export type { Rgb };
