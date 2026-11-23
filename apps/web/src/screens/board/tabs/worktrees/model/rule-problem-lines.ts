/* @layer renderer-app @kind logic */
import type { WorktreeRule, WorktreesMeta } from '../../../../../data/worktrees';

const reasonOf = (rule: WorktreeRule): string => {
  if (rule.invalid) return `invalid, ${rule.invalid}`;
  if (rule.error) return rule.error;
  return 'not found on this machine';
};

const ruleProblemLines = (meta: WorktreesMeta | null): string[] =>
  (meta?.rules ?? []).filter((rule) => rule.exists === false || Boolean(rule.error)).map((rule) => `${rule.path}: ${reasonOf(rule)}`);

export { ruleProblemLines };
