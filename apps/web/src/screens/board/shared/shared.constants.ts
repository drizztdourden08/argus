/* @layer renderer-app @kind constants */
import type { StatusTone } from '@drizztdourden08/tessera';
import type { SoftKind } from '../../../data/kanban';

const UNDO_TOAST_MS = 8000;

const ACTIVITY_TONES: Readonly<Record<string, StatusTone>> = { working: 'success', waiting: 'warning', blocked: 'danger' };

const NOW_TICK_MS = 15_000;

const SECOND_TICK_MS = 1000;

const FOCUS_KEY = 'coord.focus';

const PLACE_FIELDS =['before', 'after'];

const SOFT_LABEL: Record<SoftKind, string> = {
  archive: 'Archived',
  unarchive: 'Unarchived',
  delete: 'Deleted',
  restore: 'Restored',
};

export { ACTIVITY_TONES, FOCUS_KEY, NOW_TICK_MS, PLACE_FIELDS, SECOND_TICK_MS, SOFT_LABEL, UNDO_TOAST_MS };
