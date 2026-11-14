/* @layer renderer-app @kind constants */
import { WIDE_KEY } from '../../../shared';
import type { Option } from '../../../shared';
import { AGENT_PREFIX, OWNER, OWNER_TITLE } from '../../../../../shell/records/behavior/look.constants';
import type { ChildCount, ItemGroup, ItemOrder, KanbanFilters, KanbanPrefs } from './model.type';

const MS_PER_HOUR = 3_600_000;

const QUIET_HOURS: Record<string, number> = { doing: 4, review: 6, todo: 24 };

const STATUS_COLUMNS: Option[] = [
  { value: 'todo', label: 'To do' },
  { value: 'doing', label: 'Doing' },
  { value: 'review', label: 'Review' },
  { value: 'done', label: 'Done' },
];

const STATUS_OPTIONS: Option[] = [...STATUS_COLUMNS, { value: 'dropped', label: 'Dropped' }];

const CLOSED_STATUSES = ['done', 'dropped'];

const GENERAL = 'general';

const DEFAULT_TYPE = 'task';

const NO_PARENT = 'no-parent';

const NO_PARENT_TITLE = 'No parent';

const TRAIL_MIN_DEPTH = 3;

const CHIP_PREFIX: Readonly<Record<string, string>> = { cost: 'cost ', risk: 'risk ' };

const NO_CHILDREN: ChildCount = { done: 0, total: 0, items: [] };

const WITH_CHILDREN: readonly string[] = ['children', 'children-ac'];

const WITH_CRITERIA: readonly string[] = ['ac', 'children-ac'];

const SETTLED_DEPS: readonly string[] = ['dropped', 'gone', 'cancelled'];

const RUN_WORDS: readonly string[] = ['pass', 'fail', 'partial', 'blocked', 'skip'];

const GROUP_OPTIONS: { id: ItemGroup; label: string }[] = [
  { id: 'status', label: 'Status' },
  { id: 'thread', label: 'Thread' },
  { id: 'assignee', label: 'Assignee' },
  { id: 'parent', label: 'Parent' },
];

const ORDER_OPTIONS: { id: ItemOrder; label: string }[] = [
  { id: 'rank', label: 'Manual' },
  { id: 'updated', label: 'Updated' },
];

const LIMITS = { title: 80, subject: 40, description: 400, brief: 1200, criterion: 120, todo: 200 };

const OP_KINDS = ['create', 'edit', 'move', 'ac', 'archive', 'unarchive', 'delete', 'restore', 'bulk', 'purge', 'build', 'avatar', 'worktree-link', 'worktree-keep', 'launcher-link', 'launcher-retire'];

const HUMAN = 'human';


const PAUSE_KINDS = ['pause', 'resume'];

const ANSWER_KINDS = ['followup', 'decision'];

const BAD_STATUSES = ['conflict', 'refused'];

const STARTING_STATUS = 'todo';

const DEFAULT_FILTERS: KanbanFilters = { query: '', threads: [], status: '', types: [], assignees: [], needs: false, conflict: false, archived: false };

const DEFAULT_PREFS: KanbanPrefs = { group: 'status', order: 'rank', direction: null, showDone: false };

const STORE_KEYS = {
  filters: 'coord.f',
  group: 'coord.g.items',
  order: 'coord.sort',
  direction: 'coord.sortdir',
  showDone: 'coord.done2',
  showPaused: 'coord.paused',
  wide: WIDE_KEY,
};

export {
  AGENT_PREFIX,
  ANSWER_KINDS,
  BAD_STATUSES,
  CHIP_PREFIX,
  CLOSED_STATUSES,
  DEFAULT_FILTERS,
  DEFAULT_PREFS,
  DEFAULT_TYPE,
  GENERAL,
  GROUP_OPTIONS,
  HUMAN,
  LIMITS,
  MS_PER_HOUR,
  NO_CHILDREN,
  NO_PARENT,
  NO_PARENT_TITLE,
  OWNER,
  OWNER_TITLE,
  RUN_WORDS,
  TRAIL_MIN_DEPTH,
  STARTING_STATUS,
  OP_KINDS,
  ORDER_OPTIONS,
  PAUSE_KINDS,
  QUIET_HOURS,
  SETTLED_DEPS,
  STATUS_COLUMNS,
  STATUS_OPTIONS,
  STORE_KEYS,
  WITH_CHILDREN,
  WITH_CRITERIA,
};
