/* @layer renderer-app @kind constants */
import type { SoftKind } from '../../../../data/kanban';
import type { NoteKind } from '../../owner-posts';
import type { PurgeDraft, PurgeScope } from './trash';

const NOW_TICK_MS = 60_000;

const UNDO_TOAST_MS = 8000;

const ANY_STATUS = 'any';

const ANY_SUBJECT = 'any';

const ALL_THREADS = 'all';

const SOFT_LABEL: Record<SoftKind, string> = {
  archive: 'Archived',
  unarchive: 'Unarchived',
  delete: 'Deleted',
  restore: 'Restored',
};

const BOARD_LABEL = 'Work items';

const NO_ITEMS_TEXT = 'No work item yet. Threads enter their own as they take work on.';

const TYPING_SELECTOR = 'input, textarea, select, [contenteditable]';

const COMPOSE_KINDS: { value: NoteKind; label: string }[] = [
  { value: 'decision', label: 'Decision' },
  { value: 'followup', label: 'Note' },
];

const PURGE_SENT = 'Purge sent. The counts are checked, a backup is written, then the records are removed.';

const DEFAULT_PURGE: PurgeDraft = { scope: 'thread', thread: '', keep: [], status: [], idle: 30, confirm: '' };

const PURGE_SCOPES: { value: PurgeScope; label: string }[] = [
  { value: 'thread', label: 'This thread' },
  { value: 'keep', label: 'Every thread except the checked ones' },
  { value: 'status', label: 'By thread status' },
  { value: 'idle', label: 'Threads idle for more than' },
  { value: 'trash', label: 'Empty the trash (every deleted record)' },
];

const KANBAN_HASH = 'kanban';

const NOTE_WORD: Record<NoteKind, string> = { decision: 'decision', followup: 'note' };

const ORDER_HINT = 'Switch the order to Manual to reorder.';

export { ALL_THREADS, KANBAN_HASH, NOTE_WORD, DEFAULT_PURGE, PURGE_SCOPES, PURGE_SENT, ANY_STATUS, ANY_SUBJECT, BOARD_LABEL, COMPOSE_KINDS, ORDER_HINT, NO_ITEMS_TEXT, NOW_TICK_MS, SOFT_LABEL, TYPING_SELECTOR, UNDO_TOAST_MS };
