/* @layer renderer-app @kind constants */
import type { SegmentOption } from '@drizztdourden08/tessera';
import type { WorktreeFilters, WorktreeGroup, WorktreeLayout, WorktreeOrder } from './model';

const WORKTREE_KIND = 'worktree';

const REQUEST_KIND = 'removal-request';

const MAIN_TITLE = 'Main checkouts';

const WORKTREES_TAB = 'worktrees';

const WORKTREE_CARD_WIDTH = 272;

const LAYOUT_CHOICES: SegmentOption<WorktreeLayout>[] = [
  { value: 'list', icon: 'rows-3', hint: { label: 'List', description: 'One line per worktree' } },
  { value: 'cards', icon: 'layout-grid', hint: { label: 'Cards', description: 'Worktree cards that wrap' } },
];

const LAYOUT_IDS: WorktreeLayout[] = ['list', 'cards'];

const NEED_LABEL = 'Needs a decision';

const COMPACT_LABEL = 'Compact';

const GROUP_CHOICES: { id: WorktreeGroup; label: string }[] = [
  { id: 'item', label: 'Item' },
  { id: 'thread', label: 'Thread' },
  { id: 'status', label: 'Status' },
  { id: 'none', label: 'None' },
];

const ORDER_CHOICES: { id: WorktreeOrder; label: string }[] = [
  { id: 'need', label: 'Need first' },
  { id: 'age', label: 'Age' },
  { id: 'path', label: 'Path' },
  { id: 'size', label: 'Size' },
];

const KIND_OPTIONS = [
  { value: 'worktree', label: 'Worktree' },
  { value: 'main', label: 'Main checkout' },
  { value: 'leftover', label: 'Leftover folder' },
];

const WORKTREE_VIEW = {
  groups: GROUP_CHOICES.map((choice) => choice.id),
  orders: ORDER_CHOICES.map((choice) => choice.id),
  group: 'none' as WorktreeGroup,
  order: 'need' as WorktreeOrder,
  direction: 'desc' as const,
  filters: { query: '', threads: [], statuses: [], kinds: [] } as WorktreeFilters,
};

const NO_THREAD_TITLE = 'No thread';

const REQUESTS_TITLE = 'Removal requests from threads';

const REMOVED_TITLE = 'Removed by the worktree hygiene';

const REMOVED_CHIP = 'removed';


const STALE_TEXT = 'This changed since you opened this confirm. Close it and open it again to read what is at stake now.';

const ASK_BUTTONS: readonly { kind: 'delete' | 'check'; label: string; tip: (thread: string) => string }[] = [
  { kind: 'delete', label: 'Ask the thread to delete', tip: (thread) => `Ask ${thread} to delete this worktree: the thread decides and answers here.` },
  { kind: 'check', label: 'Check', tip: (thread) => `Ask ${thread} to check on this worktree.` },
];

const ASK_HINTS = {
  delete: 'A short message for the thread (optional): why it can go',
  check: 'A short message for the thread (optional): what to check',
} as const;

export {
  ASK_BUTTONS, ASK_HINTS, COMPACT_LABEL, GROUP_CHOICES, KIND_OPTIONS, LAYOUT_CHOICES, LAYOUT_IDS, MAIN_TITLE, NEED_LABEL, NO_THREAD_TITLE, ORDER_CHOICES, REMOVED_CHIP, REMOVED_TITLE,
  REQUEST_KIND, REQUESTS_TITLE, STALE_TEXT, WORKTREE_CARD_WIDTH, WORKTREE_KIND, WORKTREE_VIEW, WORKTREES_TAB,
};
