/* @layer renderer-app @kind constants */
import { FOCUS_KEY, WIDE_KEY } from '../../../shared';
import type { Option } from '../../../shared';
import { OWNER, OWNER_TITLE } from '../../../../../shell/records/behavior/look.constants';
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

const NO_CHILDREN: ChildCount = { done: 0, total: 0 };

const RUN_WORDS: readonly string[] = ['pass', 'fail', 'partial', 'blocked', 'skip'];

const GROUP_OPTIONS: { value: ItemGroup; label: string }[] = [
  { value: 'status', label: 'Status' },
  { value: 'thread', label: 'Thread' },
  { value: 'assignee', label: 'Assignee' },
  { value: 'parent', label: 'Parent' },
];

const ORDER_OPTIONS: { value: ItemOrder; label: string }[] = [
  { value: 'rank', label: 'Manual' },
  { value: 'updated', label: 'Updated' },
];

const LIMITS = { title: 80, subject: 40, description: 400, brief: 1200, criterion: 120, todo: 200 };

const OP_KINDS = ['create', 'edit', 'move', 'ac', 'archive', 'unarchive', 'delete', 'restore', 'bulk', 'purge', 'build', 'avatar'];

const HUMAN = 'human';


const PAUSE_KINDS = ['pause', 'resume'];

const ANSWER_KINDS = ['followup', 'decision'];

const BAD_STATUSES = ['conflict', 'refused'];

const STARTING_STATUS = 'todo';

const DEFAULT_FILTERS: KanbanFilters = { query: '', threads: [], status: '', types: [], assignees: [], needs: false, conflict: false, archived: false };

const DEFAULT_PREFS: KanbanPrefs = { group: 'status', order: 'rank', showDone: false };

const STORE_KEYS = {
  filters: 'coord.f',
  group: 'coord.g.items',
  order: 'coord.sort',
  showDone: 'coord.done2',
  focus: FOCUS_KEY,
  showPaused: 'coord.paused',
  wide: WIDE_KEY,
};

export {
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
  STATUS_COLUMNS,
  STATUS_OPTIONS,
  STORE_KEYS,
};
