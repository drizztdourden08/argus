/* @layer renderer-app @kind constants */
import type { TagLook } from '@drizztdourden08/tessera';

const LEVELS = ['info', 'warn', 'error'] as const;

const QUIET_LOOK: TagLook = { variant: 'normal', color: 'neutral' };
const DANGER_LOOK: TagLook = { variant: 'urgency', color: 'danger' };
const WARNING_LOOK: TagLook = { variant: 'urgency', color: 'warning' };
const INFO_LOOK: TagLook = { variant: 'urgency', color: 'info' };

const KIND_LOOK: Readonly<Record<string, TagLook>> = {
  fault: DANGER_LOOK,
  process: WARNING_LOOK,
  kill: DANGER_LOOK,
  close: QUIET_LOOK,
  run: INFO_LOOK,
  wake: { variant: 'category', color: 'violet' },
  error: DANGER_LOOK,
  note: QUIET_LOOK,
};

const LEVEL_LOOK: Readonly<Record<(typeof LEVELS)[number], TagLook>> = { info: QUIET_LOOK, warn: WARNING_LOOK, error: DANGER_LOOK };

const KNOWN_KINDS = Object.keys(KIND_LOOK);

export { KIND_LOOK, KNOWN_KINDS, LEVEL_LOOK, LEVELS, QUIET_LOOK };
