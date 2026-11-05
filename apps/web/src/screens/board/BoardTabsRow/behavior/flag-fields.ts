/* @layer renderer-app @kind logic */
import type { FlagFilters, TabField } from '../BoardTabsRow.type';

const flagFields = <F extends FlagFilters>(): TabField<F>[] => [
  { kind: 'flag', path: 'needs', label: 'Needs you', read: (filters) => filters.needs, write: (needs) => ({ needs }) as Partial<F> },
  { kind: 'flag', path: 'conflict', label: 'Conflict', read: (filters) => filters.conflict, write: (conflict) => ({ conflict }) as Partial<F> },
  { kind: 'flag', path: 'archived', label: 'Archived', read: (filters) => filters.archived, write: (archived) => ({ archived }) as Partial<F> },
];

export { flagFields };
