/* @layer renderer-app @kind constants */
const TAB_FIELDS: readonly string[] = ['cases', 'runs'];

const CASE_FACT_FIELDS: readonly string[] = ['mode', 'bundle'];

const CASE_KIND = 'test-case';

const NO_CHOICE = '(none)';

const CRUMB_MAX = 6;

const TAB_STORE = 'coord.pane.tab';

const TAB_ALIASES: Readonly<Record<string, string>> = { builds: 'work' };

const FACT_HIDDEN: readonly string[] = ['id', 'rev', 'deletedAt', 'seq'];

const FACT_MAX = 40;

const NAMING_FIELDS: Readonly<Record<string, string>> = { items: 'type', entries: 'kind' };

const EXTRA_STORES: Readonly<Record<string, string>> = { agents: 'agents', launchers: 'launchers', worktrees: 'worktrees', prs: 'prs', runs: 'runs' };

const FALLBACK_STORE = 'items';

const DRAWER_LABELS: Readonly<Record<string, string>> = { items: 'Work item', entries: 'Entry detail', agents: 'Agent detail', tests: 'Test detail' };

const DRAWER_FALLBACK = 'Record detail';

const TITLE_FIELDS: readonly string[] = ['title', 'text', 'name', 'task', 'branch'];

const LEAF_KINDS: readonly string[] = ['question', 'decision', 'test', 'todo', 'request', 'legacy'];

const ENTRY_LEAVES: readonly string[] = ['question', 'decision'];

const EDGE_WORDS: Readonly<Record<string, string>> = {
  item: 'blocked by',
  test: 'test',
  todo: 'to-do',
  ask: 'question',
  decision: 'decision',
  'dep-waits': 'request',
  'dep-blocked': 'blocked request',
  legacy: 'note',
};

export { CASE_FACT_FIELDS, CASE_KIND, CRUMB_MAX, DRAWER_FALLBACK, DRAWER_LABELS, EDGE_WORDS, ENTRY_LEAVES, EXTRA_STORES, FACT_HIDDEN, FALLBACK_STORE, NAMING_FIELDS, FACT_MAX, LEAF_KINDS, NO_CHOICE, TAB_ALIASES, TAB_FIELDS, TAB_STORE, TITLE_FIELDS };
