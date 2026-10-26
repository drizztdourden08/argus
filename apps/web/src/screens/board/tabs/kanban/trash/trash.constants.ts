/* @layer renderer-app @kind constants */
const TRASH_KINDS: Record<string, string> = { items: 'item', tests: 'test', actions: 'to-do', entries: 'entry', agents: 'agent' };

const TRASH_COLLECTIONS = ['items', 'tests', 'actions', 'entries', 'agents'] as const;

const PURGE_LABEL: Record<string, string> = {
  threads: 'threads',
  items: 'items',
  ac: 'criteria',
  tests: 'tests',
  entries: 'entries',
  actions: 'to-dos',
  logLines: 'log lines',
};

const PURGE_STATUSES = ['done', 'archived', 'placeholder', 'dispatched'];

const PURGE_SETTLED = ['applied', 'refused'];

const MS_PER_DAY = 86_400_000;

export { MS_PER_DAY, PURGE_LABEL, PURGE_SETTLED, PURGE_STATUSES, TRASH_COLLECTIONS, TRASH_KINDS };
