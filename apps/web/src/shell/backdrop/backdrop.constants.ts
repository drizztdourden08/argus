/* @layer renderer-app @kind constants */
const COLOR_TOKENS = ['--c-primary', '--c-secondary', '--c-tertiary', '--c-primary-dim', '--c-bg'] as const;

const BASE_TOKEN = '--c-bg';

const FALLBACK_COLORS: readonly string[] = ['#1a2433', '#2a1f3a', '#16313a', '#1f2a44', '#0d1118'];

const DARKEN_AMOUNT = 0.55;
const TRANSITION_MS = 800;

const MOTION = { speed: 0.35, distortion: 0.8, swirl: 0.25 } as const;
const STILL_MOTION = { speed: 0, distortion: 0.8, swirl: 0.25 } as const;

const PALETTE_ATTRIBUTE = 'data-palette';

export { BASE_TOKEN, COLOR_TOKENS, DARKEN_AMOUNT, FALLBACK_COLORS, MOTION, PALETTE_ATTRIBUTE, STILL_MOTION, TRANSITION_MS };
