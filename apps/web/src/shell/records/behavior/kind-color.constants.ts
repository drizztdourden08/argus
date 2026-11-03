/* @layer renderer-app @kind constants */
const TONE_COLORS: Readonly<Record<string, string>> = {
  danger: 'var(--c-danger)',
  warning: 'var(--c-warning)',
  info: 'var(--c-info)',
  success: 'var(--c-success)',
};

const NEUTRAL_COLOR = 'var(--c-text-dim)';

export { NEUTRAL_COLOR, TONE_COLORS };
