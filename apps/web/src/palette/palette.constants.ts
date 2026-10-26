/* @layer renderer-app @kind constants */
const PALETTES = ['brock', 'rotp', 'archipelia', 'bxel'] as const;

const BASIC = 'basic';

const KEY_PALETTES: Readonly<Record<string, (typeof PALETTES)[number]>> = {
  rotp: 'rotp',
  brock: 'brock',
  archipelia: 'archipelia',
  pixelarttool: 'bxel',
};

export { BASIC, KEY_PALETTES, PALETTES };
