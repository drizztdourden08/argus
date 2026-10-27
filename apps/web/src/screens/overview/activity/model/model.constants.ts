/* @layer renderer-app @kind constants */
import type { IconName, TextProps } from '@drizztdourden08/tessera';

const LEVELS = ['info', 'warn', 'error'] as const;

const KIND_ICON: Readonly<Record<string, IconName>> = {
  fault: 'triangle-alert',
  process: 'cpu',
  kill: 'circle-x',
  close: 'x',
  run: 'play',
  wake: 'bell',
  error: 'triangle-alert',
  note: 'file-text',
};

const FALLBACK_ICON: IconName = 'file-text';

const LEVEL_TONE: Readonly<Record<(typeof LEVELS)[number], TextProps['tone']>> = { info: 'info', warn: 'warning', error: 'danger' };

const KNOWN_KINDS = Object.keys(KIND_ICON);

export { FALLBACK_ICON, KIND_ICON, KNOWN_KINDS, LEVEL_TONE, LEVELS };
