/* @layer renderer-app @kind constants */
const STALE_MINUTES = 60;
const MS_PER_MINUTE = 60000;
const ATTENTION_DECISION = /needs a decision/;
const RUNS_REFRESH_MS = 250;
const OWNER_MAX = 50000;

const EMPTY_COUNTS = {
  threads: 0,
  working: 0,
  blocked: 0,
  needsYou: 0,
  itemsOpen: 0,
  doing: 0,
  review: 0,
  testsPass: 0,
  tests: 0,
  testsFailing: 0,
  agents: 0,
  paused: 0,
};

const REGISTRY_RANK: Readonly<Record<string, number>> = { fault: 0, waking: 1, listening: 2, busy: 2, idle: 3 };
const REGISTRY_RANK_FALLBACK = 4;

export {
  ATTENTION_DECISION, EMPTY_COUNTS, MS_PER_MINUTE, OWNER_MAX, REGISTRY_RANK, REGISTRY_RANK_FALLBACK, RUNS_REFRESH_MS, STALE_MINUTES,
};
