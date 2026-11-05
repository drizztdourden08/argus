/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import { toMillis } from '../../../../../format';
import type { ChangesOrder, PullRequestDoc } from '../changes.type';

const stamp = (value: string | null | undefined): number => {
  const at = value ? toMillis(value) : 0;
  return Number.isFinite(at) ? at : 0;
};

const rank = (pr: PullRequestDoc): number => (pr.status === 'open' ? 0 : 1);

const compare = (order: ChangesOrder, left: PullRequestDoc, right: PullRequestDoc): number => {
  if (order === 'title') return left.title.localeCompare(right.title);
  if (order === 'merged') return stamp(left.mergedAt) - stamp(right.mergedAt);
  return stamp(left.createdAt) - stamp(right.createdAt);
};

const sortPrs = (prs: readonly PullRequestDoc[], order: ChangesOrder = 'created', direction: SortDirection = 'desc'): PullRequestDoc[] => {
  const sign = direction === 'asc' ? 1 : -1;
  const lead = (left: PullRequestDoc, right: PullRequestDoc) => (order === 'created' ? rank(left) - rank(right) : 0);
  return [...prs].sort((left, right) => lead(left, right) || sign * compare(order, left, right));
};

export { sortPrs };
