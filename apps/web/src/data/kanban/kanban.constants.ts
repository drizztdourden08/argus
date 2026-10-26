/* @layer renderer-app @kind constants */
import type { SoftKind } from './kanban.type';

const BULK_CHUNK = 50;

const SOFT_INVERSE: Record<SoftKind, SoftKind> = {
  archive: 'unarchive',
  unarchive: 'archive',
  delete: 'restore',
  restore: 'delete',
};

const CONFLICT_REASON = 'a thread changed it after you looked';

const REFUSED_REASON = 'the board did not accept it';

const UNREACHABLE_MESSAGE = 'Dashboard unreachable, retrying.';

const COPY_FIELDS = ['kind', 'target', 'set', 'to', 'do', 'n', 'm', 'text', 'prev', 'reason'] as const;

export { BULK_CHUNK, CONFLICT_REASON, COPY_FIELDS, REFUSED_REASON, SOFT_INVERSE, UNREACHABLE_MESSAGE };
