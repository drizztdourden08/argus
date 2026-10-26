/* @layer renderer-app @kind logic */
import { needsLook } from './needs-look';
import type { ProcFilters, ProcItem } from './model.type';

const filterProcs = (procs: readonly ProcItem[], filters: ProcFilters): ProcItem[] =>
  procs.filter((proc) => (filters.status !== 'look' || needsLook(proc)) && (!filters.rule || proc.rule === filters.rule));

export { filterProcs };
