/* @layer renderer-app @kind constants */
import type { SoftKind } from '../../../data/kanban';

const UNDO_TOAST_MS = 8000;

const PLACE_FIELDS = ['before', 'after'];

const SOFT_LABEL: Record<SoftKind, string> = {
  archive: 'Archived',
  unarchive: 'Unarchived',
  delete: 'Deleted',
  restore: 'Restored',
};

export { PLACE_FIELDS, SOFT_LABEL, UNDO_TOAST_MS };
