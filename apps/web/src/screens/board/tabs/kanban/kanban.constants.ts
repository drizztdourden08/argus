/* @layer renderer-app @kind constants */
import type { TagUrgencyColor, TesseraOverrides } from '@drizztdourden08/tessera';
import type { NoteKind } from '../../owner-posts';
import type { PurgeDraft, PurgeScope } from './trash';

const NOW_TICK_MS = 60_000;

const ANY_STATUS = 'any';

const BOARD_LABEL = 'Work items';

const EMPTY_COLUMN_STRINGS: TesseraOverrides = { strings: { kanbanBoard: { empty: 'none' } } };

const STATUS_TONES: Record<string, TagUrgencyColor> = {
  todo: 'info',
  doing: 'success',
  review: 'warning',
  pending: 'warning',
  manual: 'warning',
  partial: 'warning',
  pass: 'success',
  fail: 'danger',
  blocked: 'danger',
};

const DEP_HEADINGS = { waits: 'Waiting on', asked: 'Requested by' } as const;

const NEEDS_YOU_TIP = 'A question, a to-do or a test waits on you.';

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

const GROUP_HINT = 'Group by Status or Thread to move a card across columns.';

const DONE_TIP = 'The done and dropped items are hidden until you open them. A status filter on Done or Dropped shows them too.';

const AGENT_KIND = 'agent';

const AGENT_WORK_TAB = 'work';

export { AGENT_KIND, AGENT_WORK_TAB, DEP_HEADINGS, DONE_TIP, EMPTY_COLUMN_STRINGS, GROUP_HINT, KANBAN_HASH, NOTE_WORD, DEFAULT_PURGE, PURGE_SCOPES, PURGE_SENT, STATUS_TONES, ANY_STATUS, BOARD_LABEL, COMPOSE_KINDS, ORDER_HINT, NEEDS_YOU_TIP, NO_ITEMS_TEXT, NOW_TICK_MS, TYPING_SELECTOR };
