/* @layer renderer-app @kind logic */
import { useMemo, useState } from 'react';
import { useProjectName } from '../../projects';
import { filterProcs, needsLook, ruleGroups, ruleNames, rulesOf, toProcRow } from '../model';
import type { ProcFilters } from '../model';
import { NO_FILTERS } from '../processes.constants';
import type { ProcessFeed, ProcessView } from '../processes.type';

const useProcessView = (feed: ProcessFeed): ProcessView => {
  const nameOf = useProjectName();
  const [filters, setFilters] = useState<ProcFilters>(NO_FILTERS);
  const { procs, meta } = feed;
  const shown = useMemo(() => filterProcs(procs, filters), [procs, filters]);
  const rows = useMemo(() => shown.map((proc) => toProcRow(proc, nameOf)), [shown, nameOf]);
  return {
    filters,
    setFilters,
    rows,
    look: procs.filter(needsLook).length,
    groups: ruleGroups(meta, procs),
    rules: rulesOf(meta),
    ruleNames: ruleNames(meta, procs),
  };
};

export { useProcessView };
