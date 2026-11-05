/* @layer renderer-app @kind logic */
import type { FilterClause } from '@drizztdourden08/tessera';
import type { ClauseField } from '../BoardTabsRow.type';

const valuesOf = <F>(field: Exclude<ClauseField<F>, { kind: 'flag' }>, filters: F): string[] => {
  if (field.kind === 'multi') return [...field.read(filters)];
  const value = field.read(filters);
  return value ? [value] : [];
};

const clauseOf = <F>(field: ClauseField<F>, filters: F): FilterClause | null => {
  if (field.kind === 'flag') return field.read(filters) ? { id: field.path, path: field.path, op: 'isTrue', value: null, enabled: true } : null;
  const values = valuesOf(field, filters);
  return values.length > 0 ? { id: field.path, path: field.path, op: 'anyOf', value: values, enabled: true } : null;
};

const toClauses = <F>(fields: readonly ClauseField<F>[], filters: F): FilterClause[] =>
  fields.map((field) => clauseOf(field, filters)).filter((clause): clause is FilterClause => clause !== null);

export { toClauses };
