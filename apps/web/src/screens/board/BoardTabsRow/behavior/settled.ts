/* @layer renderer-app @kind logic */
import type { FilterClause } from '@drizztdourden08/tessera';
import type { ClauseField } from '../BoardTabsRow.type';

const lastOnly = (clause: FilterClause): FilterClause =>
  Array.isArray(clause.value) && clause.value.length > 1 ? { ...clause, value: clause.value.slice(-1) } : clause;

const settled = <F>(fields: readonly ClauseField<F>[], clauses: readonly FilterClause[]): FilterClause[] =>
  clauses.map((clause) => (fields.some((field) => field.kind === 'single' && field.path === clause.path) ? lastOnly(clause) : clause));

export { settled };
