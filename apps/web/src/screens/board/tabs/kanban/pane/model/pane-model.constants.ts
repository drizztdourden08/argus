/* @layer renderer-app @kind constants */
const TAB_FIELDS: readonly string[] = ['cases', 'runs'];

const NO_CHOICE = '(none)';

const CRUMB_MAX = 6;

const TAB_STORE = 'coord.pane.tab';

const OPENABLE_REF = /^[IT]-\d+$/;

const LEAF_KINDS: readonly string[] = ['question', 'decision', 'test', 'todo', 'request', 'legacy'];

const ENTRY_LEAVES: readonly string[] = ['question', 'decision'];

const ENTRY_CAUSES: readonly string[] = ['ask', 'decision'];

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

export { CRUMB_MAX, EDGE_WORDS, ENTRY_CAUSES, ENTRY_LEAVES, LEAF_KINDS, NO_CHOICE, OPENABLE_REF, TAB_FIELDS, TAB_STORE };
