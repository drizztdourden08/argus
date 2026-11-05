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

const DEFAULT_FILTERS: TestFilters = { query: '', threads: [], builds: [], nobuild: false, status: '', subject: '', plan: '', needs: false, conflict: false, archived: false };

const DEFAULT_PREFS: TestPrefs = { group: 'plan', order: 'rank', direction: null, showPassed: true, showClosed: false };

const STORE_KEYS = { filters: 'coord.f', group: 'coord.g.tests4', direction: 'coord.sortdir.tests', passed: 'coord.passed', closed: 'coord.closed', paused: 'coord.paused' };

const GROUP_OPTIONS: { id: TestGroup; label: string }[] = [
  { id: 'plan', label: 'Plan' },
  { id: 'build', label: 'Bundle' },
  { id: 'item', label: 'Item' },
  { id: 'thread', label: 'Thread' },
];

const ORDER_OPTIONS: { id: TestOrder; label: string }[] = [
  { id: 'rank', label: 'Manual' },
  { id: 'updated', label: 'Updated' },
];

const STATUS_FILTERS = [
  { value: 'pending', label: 'To run' },
  { value: 'fail', label: 'Failed' },
  { value: 'partial', label: 'Partial' },
  { value: 'blocked', label: 'Blocked' },
  { value: 'pass', label: 'Passed' },
];

const EDIT_LIMITS = { title: 80, covers: 80, build: 40, mode: 8 };

const BAD_STATUSES = ['conflict', 'refused'];

const PLAN_CLOSED_STAGES = ['passed', 'abandoned'];

const PLAN_KIND = 'test-plan';

const OWNER_ASSIGNEE = 'owner';

const AUTO_TIP = 'A thread can run this one itself; you can still post a result';

const OP_KINDS = ['create', 'edit', 'move', 'ac', 'archive', 'unarchive', 'delete', 'restore', 'bulk', 'purge', 'build', 'avatar', 'worktree-link', 'worktree-keep', 'launcher-link', 'launcher-retire'];

export {
  AUTO_TIP, BAD_STATUSES, DEFAULT_FILTERS, DEFAULT_PREFS, DONE_FOLD_AFTER, EDIT_LIMITS, GROUP_OPTIONS, HUMAN, NO_BUNDLE, NO_KEY, NOTE_MIN_CHARS, NOTE_OUTCOMES, OP_KINDS, OWNER_ASSIGNEE,
  PASSED_STATUSES, PLAN_CLOSED_STAGES, PLAN_KIND, ORDER_OPTIONS, STATUS_FILTERS, STORE_KEYS, WAKE_WAIT_MS,
};
