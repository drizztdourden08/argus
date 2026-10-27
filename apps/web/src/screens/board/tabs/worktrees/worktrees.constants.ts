/* @layer renderer-app @kind constants */
import type { FieldDescriptor, TableColumn } from '@drizztdourden08/tessera';
import type { WorktreeFilter } from './model';

const FILTER_KEY = 'coord.wtfilter';

const RULES_KEY = 'coord.wtrules';

const FILTER_IDS: readonly WorktreeFilter[] = ['all', 'need', 'left'];

const RULES_STATES: readonly ('0' | '1')[] = ['0', '1'];

const SCHEMA: readonly FieldDescriptor[] = [
  { path: 'path', label: 'Path', kind: 'string', optional: false },
  { path: 'status', label: 'Status', kind: 'string', optional: false },
  { path: 'branch', label: 'Branch', kind: 'string', optional: false },
  { path: 'touched', label: 'Touched', kind: 'string', optional: false },
  { path: 'ahead', label: 'Ahead / behind', kind: 'number', optional: false },
  { path: 'dirty', label: 'Dirty', kind: 'number', optional: false },
  { path: 'subject', label: 'Last commit', kind: 'string', optional: false },
  { path: 'request', label: 'Request', kind: 'string', optional: false },
  { path: 'actions', label: 'Actions', kind: 'string', optional: false },
];

const FALLBACK_COLUMNS: readonly TableColumn[] = [
  { path: 'path', width: 310 },
  { path: 'status', width: 110 },
  { path: 'branch', width: 160 },
  { path: 'touched', width: 80 },
  { path: 'ahead', width: 104 },
  { path: 'dirty', width: 46 },
  { path: 'subject', grow: true },
  { path: 'request', width: 100 },
  { path: 'actions', width: 170 },
];

const EXPLAINER_TEXT = 'Where the thread\'s worktrees are, as git sees them now. The thread decides what to do, or you delete one here (a backup is saved first when work would be lost).';

const NO_RULES_HINT = 'The registered thread adds the places once: coord worktree rule add <path> (--kind folder: every sub-folder is a worktree, --kind repo: the repository\'s git worktree list). Argus then looks with read-only git and wakes the thread when one goes stale.';

const FILTER_LABEL = 'Show';

const COUNT_LABEL: readonly [string, string] = ['row', 'rows'];

const STALE_TEXT = 'This changed since you opened this confirm. Close it and open it again to read what is at stake now.';

const ASK_BUTTONS: readonly { kind: 'delete' | 'check'; label: string; tip: (thread: string) => string }[] = [
  { kind: 'delete', label: 'Ask the thread to delete', tip: (thread) => `Ask ${thread} to delete this worktree: the thread decides and answers here.` },
  { kind: 'check', label: 'Check', tip: (thread) => `Ask ${thread} to check on this worktree.` },
];

const ASK_HINTS = {
  delete: 'A short message for the thread (optional): why it can go',
  check: 'A short message for the thread (optional): what to check',
} as const;

export { ASK_BUTTONS, ASK_HINTS, COUNT_LABEL, EXPLAINER_TEXT, FALLBACK_COLUMNS, FILTER_IDS, FILTER_KEY, FILTER_LABEL, NO_RULES_HINT, RULES_KEY, RULES_STATES, SCHEMA, STALE_TEXT };
