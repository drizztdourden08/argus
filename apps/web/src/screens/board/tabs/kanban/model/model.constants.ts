/* @layer renderer-app @kind constants */
import type { ItemGroup, ItemOrder, KanbanFilters, KanbanPrefs, Option } from './model.type';

const MS_PER_SECOND = 1000;

const SECONDS_PER_MINUTE = 60;

const MINUTES_PER_HOUR = 60;

const MS_PER_HOUR = 3_600_000;

const HOURS_AS_DAYS_FROM = 48;

const HOURS_PER_DAY = 24;

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

const GROUP_OPTIONS: { value: ItemGroup; label: string }[] = [
  { value: 'status', label: 'Status' },
  { value: 'subject', label: 'Subject' },
  { value: 'thread', label: 'Thread' },
];

const ORDER_OPTIONS: { value: ItemOrder; label: string }[] = [
  { value: 'rank', label: 'Manual' },
  { value: 'updated', label: 'Updated' },
];

const LIMITS = { title: 80, subject: 40, description: 400, criterion: 120, todo: 200 };

const OP_KINDS = ['create', 'edit', 'move', 'ac', 'archive', 'unarchive', 'delete', 'restore', 'bulk', 'purge', 'build'];

const HUMAN = 'human';

const ITEM_HASH = /^[IH]-\d+$/;

const PAUSE_KINDS = ['pause', 'resume'];

const ANSWER_KINDS = ['followup', 'decision'];

const PLACE_FIELDS = ['before', 'after'];

const BAD_STATUSES = ['conflict', 'refused'];

const STARTING_STATUS = 'todo';

const DEFAULT_FILTERS: KanbanFilters = { query: '', threads: [], status: '', subject: '', needs: false, conflict: false, archived: false };

const DEFAULT_PREFS: KanbanPrefs = { group: 'status', order: 'rank', showDone: false };

const STORE_KEYS = {
  filters: 'coord.f',
  group: 'coord.g.items',
  order: 'coord.sort',
  showDone: 'coord.done2',
  focus: 'coord.focus',
  showPaused: 'coord.paused',
  wide: 'coord.drawer.wide',
};

export {
  ANSWER_KINDS,
  BAD_STATUSES,
  CLOSED_STATUSES,
  DEFAULT_FILTERS,
  DEFAULT_PREFS,
  GENERAL,
  GROUP_OPTIONS,
  HOURS_AS_DAYS_FROM,
  HOURS_PER_DAY,
  HUMAN,
  LIMITS,
  MINUTES_PER_HOUR,
  MS_PER_HOUR,
  MS_PER_SECOND,
  STARTING_STATUS,
  OP_KINDS,
  ORDER_OPTIONS,
  ITEM_HASH,
  PAUSE_KINDS,
  PLACE_FIELDS,
  QUIET_HOURS,
  SECONDS_PER_MINUTE,
  STATUS_COLUMNS,
  STATUS_OPTIONS,
  STORE_KEYS,
};
