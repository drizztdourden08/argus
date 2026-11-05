/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { ChangesFilters, ChangesOrder, PullRequestDoc } from '../changes.type';
import { sortPrs } from './sort-prs';

const matchesQuery = (pr: PullRequestDoc, query: string): boolean => {
  const needle = query.trim().toLowerCase();
  return needle === '' || pr.title.toLowerCase().includes(needle) || pr.head.toLowerCase().includes(needle);
};

const matchesFilters = (pr: PullRequestDoc, filters: ChangesFilters): boolean =>
  matchesQuery(pr, filters.query)
  && (filters.status === '' || pr.status === filters.status)
  && (filters.item === '' || pr.item === filters.item)
  && (filters.author === '' || pr.createdBy === filters.author)
  && (filters.base === '' || pr.base === filters.base);

const viewPrs = (prs: readonly PullRequestDoc[], filters: ChangesFilters, order: ChangesOrder, direction: SortDirection): PullRequestDoc[] =>
  sortPrs(prs.filter((pr) => matchesFilters(pr, filters)), order, direction);

export { viewPrs };
