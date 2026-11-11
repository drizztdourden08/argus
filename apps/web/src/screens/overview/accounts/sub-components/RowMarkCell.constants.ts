/* @layer renderer-app @kind constants */
import type { ChipLook } from '../compare.type';

const MARK_LOOK: Readonly<Record<'running' | 'done' | 'failed', ChipLook>> = {
  running: { label: 'running', tone: 'info' },
  done: { label: 'done', tone: 'success' },
  failed: { label: 'failed', tone: 'danger' },
};

export { MARK_LOOK };
