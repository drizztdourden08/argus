/* @layer renderer-app @kind logic */
import type { WorktreesMeta } from '../../../../../data/worktrees';

const ruleProblems = (meta: WorktreesMeta | null): number =>
  (meta?.rules ?? []).filter((rule) => rule.exists === false || Boolean(rule.error)).length;

export { ruleProblems };
