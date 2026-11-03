/* @layer renderer-app @kind constants */
import type { WorktreeFilter } from './model';

const WORKTREE_KIND = 'worktree';

const REQUEST_KIND = 'removal-request';

const MAIN_TITLE = 'Main checkouts';

const FILTER_KEY = 'coord.wtfilter';

const RULES_KEY = 'coord.wtrules';

const FILTER_IDS: readonly WorktreeFilter[] = ['all', 'need', 'left'];

const RULES_STATES: readonly ('0' | '1')[] = ['0', '1'];

const EXPLAINER_TEXT = 'Where the thread\'s worktrees are, as git sees them now. The thread decides what to do, or you delete one here (a backup is saved first when work would be lost).';

const NO_RULES_HINT = 'The registered thread adds the places once: coord worktree rule add <path> (--kind folder: every sub-folder is a worktree, --kind repo: the repository\'s git worktree list). Argus then looks with read-only git and wakes the thread when one goes stale.';

const FILTER_LABEL = 'Show';

const REQUESTS_TITLE = 'Removal requests from threads';

const REMOVED_TITLE = 'Removed by the worktree hygiene';

const REMOVED_CHIP = 'removed';


const STALE_TEXT = 'This changed since you opened this confirm. Close it and open it again to read what is at stake now.';

const ASK_BUTTONS: readonly { kind: 'delete' | 'check'; label: string; tip: (thread: string) => string }[] = [
  { kind: 'delete', label: 'Ask the thread to delete', tip: (thread) => `Ask ${thread} to delete this worktree: the thread decides and answers here.` },
  { kind: 'check', label: 'Check', tip: (thread) => `Ask ${thread} to check on this worktree.` },
];

const ASK_HINTS = {
  delete: 'A short message for the thread (optional): why it can go',
  check: 'A short message for the thread (optional): what to check',
} as const;

export { ASK_BUTTONS, ASK_HINTS, EXPLAINER_TEXT, MAIN_TITLE, FILTER_IDS, FILTER_KEY, FILTER_LABEL, NO_RULES_HINT, REMOVED_CHIP, REMOVED_TITLE, REQUEST_KIND, REQUESTS_TITLE, RULES_KEY, RULES_STATES, STALE_TEXT, WORKTREE_KIND };
