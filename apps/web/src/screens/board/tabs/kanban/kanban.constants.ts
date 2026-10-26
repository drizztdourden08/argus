/* @layer renderer-app @kind constants */
import type { SoftKind } from '../../../../data/kanban';

const NOW_TICK_MS = 60_000;

const UNDO_TOAST_MS = 8000;

const ANY_STATUS = 'any';

const ANY_SUBJECT = 'any';

const SOFT_LABEL: Record<SoftKind, string> = {
  archive: 'Archived',
  unarchive: 'Unarchived',
  delete: 'Deleted',
  restore: 'Restored',
};

const BOARD_LABEL = 'Work items';

const NO_ITEMS_TEXT = 'No work item yet. Threads enter their own as they take work on.';

const ORDER_HINT = 'Switch the order to Manual to reorder.';

export { ANY_STATUS, ANY_SUBJECT, BOARD_LABEL, ORDER_HINT, NO_ITEMS_TEXT, NOW_TICK_MS, SOFT_LABEL, UNDO_TOAST_MS };
