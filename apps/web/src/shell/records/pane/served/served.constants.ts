/* @layer renderer-app @kind constants */
const BUILDS_TITLE = 'Builds';

const WORKTREES_TITLE = 'Worktrees';

const NO_BUILDS_TEXT = 'No build serves this item.';

const NO_WORKTREES_TEXT = 'No worktree is linked to this item.';

const SOURCE_TIP: Readonly<Record<string, string>> = {
  manual: 'Linked by hand (coord worktree link)',
  agent: 'The item of the agent that owns the worktree',
  branch: 'An item id in the branch name',
  text: 'An open item names this worktree',
};

export { BUILDS_TITLE, NO_BUILDS_TEXT, NO_WORKTREES_TEXT, SOURCE_TIP, WORKTREES_TITLE };
