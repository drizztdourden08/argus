/* @layer renderer-app @kind constants */
import type { LogKindTone } from '@drizztdourden08/tessera';

const LEVELS = ['info', 'warn', 'error'] as const;

const KINDS = ['fault', 'process', 'kill', 'close', 'run', 'wake', 'error', 'note'] as const;

const KIND_TONE: Readonly<Record<string, LogKindTone>> = {
  fault: 'danger',
  process: 'warning',
  kill: 'danger',
  close: 'muted',
  run: 'info',
  wake: 'primary',
  error: 'danger',
  note: 'muted',
};

const KIND_FALLBACK_TONE: LogKindTone = 'muted';

export { KIND_FALLBACK_TONE, KIND_TONE, KINDS, LEVELS };
