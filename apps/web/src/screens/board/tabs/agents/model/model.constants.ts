/* @layer renderer-app @kind constants */
import type { ModelColor, ModelFamily, StatusLook } from './model.type';

const ACTIVE_STATUSES: readonly string[] = ['working', 'waiting', 'stop-requested'];

const MS_PER_MINUTE = 60_000;

const STALE_MS = 45 * MS_PER_MINUTE;

const MINUTES_PER_HOUR = 60;

const HOURS_PER_DAY = 24;

const HOURS_AS_DAYS_FROM = 48;

const FILE_TOOLS: readonly string[] = ['Edit', 'Write', 'Read', 'MultiEdit', 'NotebookEdit'];

const BUSY_ACTIVITIES: readonly string[] = ['tool', 'thinking'];

const STATUS_LOOKS: Readonly<Record<string, StatusLook>> = {
  working: { label: 'working', tone: 'success' },
  waiting: { label: 'waiting', tone: 'warning' },
  'stop-requested': { label: 'stopping', tone: 'danger' },
  done: { label: 'done', tone: 'neutral' },
  stopped: { label: 'stopped', tone: 'neutral' },
  failed: { label: 'failed', tone: 'danger' },
};

const MODEL_FAMILIES: readonly ModelFamily[] = ['haiku', 'sonnet', 'opus'];

const MODEL_COLORS: Readonly<Record<ModelFamily, ModelColor>> = {
  haiku: 'neutral',
  sonnet: 'primary',
  opus: 'tertiary',
  other: 'secondary',
};

const CHAT_ORDER: Readonly<Record<string, number>> = { blocked: 0, active: 1, waiting: 2, idle: 3, dispatched: 4, paused: 5 };

const CONFLICT_TEXT = 'a thread changed it after you looked';

const REFUSED_TEXT = 'the board did not accept it';

const EMPTY_ALL = 'No sub-agent yet. The lead starts them as it works and they show here.';

const EMPTY_RUNNING = 'No sub-agent is working right now.';

const EMPTY_MATCH = 'No agent matches.';

const UNREACHABLE_TEXT = 'Dashboard unreachable, retrying.';

const NO_WHY = 'no justification recorded';

const MESSAGE_CLIP = 200;

export {
  ACTIVE_STATUSES,
  BUSY_ACTIVITIES,
  CHAT_ORDER,
  CONFLICT_TEXT,
  EMPTY_ALL,
  EMPTY_MATCH,
  EMPTY_RUNNING,
  FILE_TOOLS,
  HOURS_AS_DAYS_FROM,
  HOURS_PER_DAY,
  MESSAGE_CLIP,
  MINUTES_PER_HOUR,
  MODEL_COLORS,
  MODEL_FAMILIES,
  MS_PER_MINUTE,
  NO_WHY,
  REFUSED_TEXT,
  STALE_MS,
  STATUS_LOOKS,
  UNREACHABLE_TEXT,
};
