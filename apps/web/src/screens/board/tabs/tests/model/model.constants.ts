/* @layer renderer-app @kind constants */
import type { TestFilters, TestGroup, TestOrder, TestPrefs } from './model.type';

const NO_BUNDLE = '(no build)';

const NOTE_MIN_CHARS = 3;

const DONE_FOLD_AFTER = 5;

const NOTE_OUTCOMES = ['fail', 'partial', 'blocked'];

const WAKE_WAIT_MS = 20000;

const HUMAN = 'human';

const PASSED_STATUSES = ['pass', 'moved'];

const NO_KEY = 'none';

const DEFAULT_FILTERS: TestFilters = { query: '', threads: [], builds: [], nobuild: false, status: '', subject: '', needs: false, conflict: false, archived: false };

const DEFAULT_PREFS: TestPrefs = { group: 'build', order: 'rank', showPassed: true, showClosed: false };

const STORE_KEYS = { filters: 'coord.f', group: 'coord.g.tests3', passed: 'coord.passed', closed: 'coord.closed', wide: 'coord.drawer.wide' };

const GROUP_OPTIONS: { value: TestGroup; label: string }[] = [
  { value: 'build', label: 'Bundle' },
  { value: 'item', label: 'Item' },
  { value: 'thread', label: 'Thread' },
];

const ORDER_OPTIONS: { value: TestOrder; label: string }[] = [
  { value: 'rank', label: 'Manual' },
  { value: 'updated', label: 'Updated' },
];

const STATUS_FILTERS = [
  { value: 'pending', label: 'To run' },
  { value: 'fail', label: 'Failed' },
  { value: 'partial', label: 'Partial' },
  { value: 'blocked', label: 'Blocked' },
  { value: 'pass', label: 'Passed' },
];

const EDIT_LIMITS = { title: 80, covers: 80, build: 40, mode: 8 };

const BAD_STATUSES =['conflict', 'refused'];

const OP_KINDS = ['create', 'edit', 'move', 'ac', 'archive', 'unarchive', 'delete', 'restore', 'bulk', 'purge', 'build'];

export {
  BAD_STATUSES, DEFAULT_FILTERS, DEFAULT_PREFS, DONE_FOLD_AFTER, EDIT_LIMITS, GROUP_OPTIONS, HUMAN, NO_BUNDLE, NO_KEY, NOTE_MIN_CHARS, NOTE_OUTCOMES, OP_KINDS,
  PASSED_STATUSES, ORDER_OPTIONS, STATUS_FILTERS, STORE_KEYS, WAKE_WAIT_MS,
};
