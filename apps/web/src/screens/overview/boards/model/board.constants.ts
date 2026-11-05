/* @layer renderer-app @kind constants */
import type { AttentionGroupId, BoardCounts } from './board.type';

const STALE_MINUTES = 60;
const MS_PER_MINUTE = 60000;
const ATTENTION_DECISION = /needs a decision/;
const OWNER_MAX = 50000;

const EMPTY_BOARD_COUNTS: BoardCounts = {
  threads: 0,
  working: 0,
  blocked: 0,
  needsYou: 0,
  itemsOpen: 0,
  doing: 0,
  review: 0,
  done: 0,
  testsPass: 0,
  tests: 0,
  testsFailing: 0,
  agents: 0,
  paused: 0,
};

const GROUP_ORDER: readonly AttentionGroupId[] = ['needs-you', 'running', 'quiet', 'not-started', 'archived'];

const GROUP_LABELS: Readonly<Record<AttentionGroupId, string>> = {
  'needs-you': 'Needs you',
  running: 'Running',
  quiet: 'Quiet',
  'not-started': 'Not started',
  archived: 'Archived',
};

const FOLDED_GROUPS: readonly AttentionGroupId[] = ['not-started', 'archived'];

export { ATTENTION_DECISION, EMPTY_BOARD_COUNTS, FOLDED_GROUPS, GROUP_LABELS, GROUP_ORDER, MS_PER_MINUTE, OWNER_MAX, STALE_MINUTES };
