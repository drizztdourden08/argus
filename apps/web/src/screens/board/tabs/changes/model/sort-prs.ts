/* @layer renderer-app @kind logic */
import { toMillis } from '../../../../../format';
import type { PullRequestDoc } from '../changes.type';

const stamp = (pr: PullRequestDoc): number => {
  const at = toMillis(pr.createdAt);
  return Number.isFinite(at) ? at : 0;
};

const rank = (pr: PullRequestDoc): number => (pr.status === 'open' ? 0 : 1);

const sortPrs = (prs: readonly PullRequestDoc[]): PullRequestDoc[] =>
  [...prs].sort((left, right) => rank(left) - rank(right) || stamp(right) - stamp(left));

export { sortPrs };
