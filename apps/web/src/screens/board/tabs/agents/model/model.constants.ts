/* @layer renderer-app @kind constants */
import type { AgentField, ModelFamily, ModelLook, StatusLook } from './model.type';

const ACTIVE_STATUSES: readonly string[] = ['working', 'waiting', 'stop-requested'];

const MS_PER_SECOND = 1000;

const MS_PER_MINUTE = 60_000;

const STALE_MS = 45 * MS_PER_MINUTE;

const MINUTES_PER_HOUR = 60;

const HOURS_PER_DAY = 24;

const HOURS_AS_DAYS_FROM = 48;

const FILE_TOOLS: readonly string[] = ['Edit', 'Write', 'Read', 'MultiEdit', 'NotebookEdit'];

const STATUS_LOOKS: Readonly<Record<string, StatusLook>> = {
  working: { label: 'working', tone: 'success' },
  waiting: { label: 'waiting', tone: 'warning' },
  'stop-requested': { label: 'stopping', tone: 'warning' },
  done: { label: 'done', tone: 'neutral' },
  stopped: { label: 'stopped', tone: 'neutral' },
  failed: { label: 'failed', tone: 'danger' },
};

const MODEL_FAMILIES: readonly ModelFamily[] = ['haiku', 'sonnet', 'opus'];

const MODEL_LOOKS: Readonly<Record<ModelFamily, ModelLook>> = {
  haiku: { tone: 'info', dot: false },
  sonnet: { tone: 'success', dot: false },
  opus: { tone: 'neutral', dot: true },
  other: { tone: 'neutral', dot: false },
};

const AGENT_LIMITS: Readonly<Record<AgentField, number>> = { task: 120, note: 100, branch: 80, worktree: 120 };

const CHAT_ORDER: Readonly<Record<string, number>> = { blocked: 0, active: 1, waiting: 2, idle: 3, dispatched: 4, paused: 5 };

const CONFLICT_TEXT = 'a thread changed it after you looked';

const REFUSED_TEXT = 'the board did not accept it';

const EMPTY_ALL = 'No sub-agent yet. The lead starts them as it works and they show here.';

const EMPTY_RUNNING = 'No sub-agent is working right now.';

const UNREACHABLE_TEXT = 'Dashboard unreachable, retrying.';

const NO_WHY = 'no justification recorded';

const ITEM_LABEL_CHARS = 40;

const NO_ITEM = 'none';

const NO_ITEM_LINE = 'No item yet. An agent works on at least one item.';

const NO_BLOCKERS_LINE = 'Nothing blocks this agent.';

const NO_RUNS_LINE = 'No run was started by this agent.';

const ADD_ITEM_LABEL = 'add an item';

const MASTER_WORD = 'master';

const LOG_PANE_TAB = 'log';

const NO_ITEM_TITLE = 'No item';

const CLOSED_ITEMS_TITLE = 'Closed items';

const NO_WORKTREE_TEXT = 'no worktree';

const NO_WORKTREE_TIP = 'No worktree or master is recorded for this agent.';

const MASTER_TIP = 'Works on the main checkout.';

const WORKTREE_FALLBACK_CODE = 'WT';

const BUILD_FALLBACK_CODE = 'BL';

const NO_RANK = 1_000_000_000;

const SECONDS_PER_MINUTE = 60;

const SECONDS_PER_HOUR = 3600;

export {
  ACTIVE_STATUSES,
  ADD_ITEM_LABEL,
  AGENT_LIMITS,
  CHAT_ORDER,
  CONFLICT_TEXT,
  EMPTY_ALL,
  EMPTY_RUNNING,
  FILE_TOOLS,
  HOURS_AS_DAYS_FROM,
  HOURS_PER_DAY,
  CLOSED_ITEMS_TITLE,
  ITEM_LABEL_CHARS,
  LOG_PANE_TAB,
  NO_ITEM,
  BUILD_FALLBACK_CODE,
  NO_BLOCKERS_LINE,
  NO_ITEM_LINE,
  NO_ITEM_TITLE,
  NO_RANK,
  NO_RUNS_LINE,
  NO_WORKTREE_TEXT,
  NO_WORKTREE_TIP,
  MASTER_TIP,
  MASTER_WORD,
  MINUTES_PER_HOUR,
  MODEL_LOOKS,
  MODEL_FAMILIES,
  MS_PER_MINUTE,
  MS_PER_SECOND,
  NO_WHY,
  REFUSED_TEXT,
  SECONDS_PER_HOUR,
  SECONDS_PER_MINUTE,
  STALE_MS,
  STATUS_LOOKS,
  UNREACHABLE_TEXT,
  WORKTREE_FALLBACK_CODE,
};
