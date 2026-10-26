/* @layer renderer-app @kind logic */
import { isManualWaiting } from './is-manual-waiting';
import { STATUS_FILTERS } from './model.constants';
import type { TestDoc, TestFilters } from './model.type';

const matchesQuery = (test: TestDoc, query: string): boolean => {
  const needle = query.toLowerCase();
  return !needle || test.id.toLowerCase().includes(needle) || test.title.toLowerCase().includes(needle);
};

const matchesStatus = (test: TestDoc, status: string): boolean =>
  !status || !STATUS_FILTERS.some((option) => option.value === status) || test.status === status;

const matchesBuild = (test: TestDoc, filters: TestFilters): boolean => {
  if (filters.builds.length === 0 && !filters.nobuild) return true;
  return test.build ? filters.builds.includes(test.build) : filters.nobuild;
};

const matchesPlace = (test: TestDoc, filters: TestFilters): boolean =>
  (filters.threads.length === 0 || filters.threads.includes(test.thread ?? '')) && (!filters.subject || (test.subject ?? '') === filters.subject);

const matchesAttention = (test: TestDoc, filters: TestFilters, conflicted: boolean): boolean =>
  (!filters.needs || isManualWaiting(test)) && (!filters.conflict || conflicted);

const matchesFilters = (test: TestDoc, filters: TestFilters, conflicted: boolean): boolean =>
  matchesQuery(test, filters.query) && matchesStatus(test, filters.status) && matchesBuild(test, filters)
  && matchesPlace(test, filters) && matchesAttention(test, filters, conflicted);

export { matchesFilters };
