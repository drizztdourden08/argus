/* @layer renderer-app @kind constants */
const COLOR_TOKENS = ['--c-primary', '--c-secondary', '--c-tertiary', '--c-primary-dim', '--c-bg'] as const;

const DEEP_TOKENS: readonly string[] = ['--c-primary-dim', '--c-bg'];

const FALLBACK_COLORS: readonly string[] = ['#1a2433', '#2a1f3a', '#16313a', '#1f2a44', '#0d1118'];

const DARKEN_AMOUNT = 0.3;
const DEEP_LIFT = 0.35;
const TRANSITION_MS = 800;

const MOTION = { speed: 0.35, distortion: 0.8, swirl: 0.25, interactive: true } as const;
const STILL_MOTION = { speed: 0, distortion: 0.8, swirl: 0.25, interactive: false } as const;

const GRAIN = { mixer: 0.6, overlay: 0.45 } as const;

const POINTER = {
  settleSeconds: 1,
  rest: 0.0005,
  center: 0.5,
} as const;

const MAX_FRAME_SECONDS = 0.1;

const SIZING_UNIFORMS = {
  u_fit: 1,
  u_rotation: 0,
  u_scale: 1,
  u_offsetX: 0,
  u_offsetY: 0,
  u_originX: 0.5,
  u_originY: 0.5,
  u_worldWidth: 0,
  u_worldHeight: 0,
} as const;

const PALETTE_ATTRIBUTE = 'data-palette';

export {
  COLOR_TOKENS,
  DARKEN_AMOUNT,
  DEEP_LIFT,
  DEEP_TOKENS,
  FALLBACK_COLORS,
  GRAIN,
  MAX_FRAME_SECONDS,
  MOTION,
  PALETTE_ATTRIBUTE,
  POINTER,
  SIZING_UNIFORMS,
  STILL_MOTION,
  TRANSITION_MS,
};
