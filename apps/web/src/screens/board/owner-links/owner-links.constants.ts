/* @layer renderer-app @kind constants */
import type { SelectOption } from '@drizztdourden08/tessera';

const BUILD_KIND = 'build';

const NO_CHOICE = 'none';

const ITEM_LABEL_CHARS = 60;

const NOTE_MAX = 300;

const DEFAULT_KEEP_DAYS = '7';

const KEEP_DAYS: readonly SelectOption[] = ['1', '3', '7', '14', '30', '90'].map((days) => ({ value: days, label: days === '1' ? '1 day' : `${days} days` }));

const WORKTREE_LINK_NOTE = 'A worktree goes with its item: when the item is done or dropped, Argus removes the worktree after a short grace and writes a backup first.';

const BUILD_LINK_NOTE = 'A build serves one open item and closes when that item closes.';

const KEEP_NOTE = 'A kept worktree is not flagged as stale until the days are over.';

const RETIRE_NOTE = 'A retired build has no Play and moves under Retired. The master build is never retired.';

export { BUILD_KIND, BUILD_LINK_NOTE, DEFAULT_KEEP_DAYS, ITEM_LABEL_CHARS, KEEP_DAYS, KEEP_NOTE, NO_CHOICE, NOTE_MAX, RETIRE_NOTE, WORKTREE_LINK_NOTE };
