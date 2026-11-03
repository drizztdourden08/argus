/* @layer renderer-app @kind constants */
import type { TextProps } from '@drizztdourden08/tessera';

const LEVELS = ['info', 'warn', 'error'] as const;

const KNOWN_KINDS: readonly string[] = ['fault', 'process', 'kill', 'close', 'run', 'wake', 'error', 'note'];

const LEVEL_TONE: Readonly<Record<(typeof LEVELS)[number], TextProps['tone']>> = { info: 'info', warn: 'warning', error: 'danger' };

export { KNOWN_KINDS, LEVEL_TONE, LEVELS };
