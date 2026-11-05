/* @layer renderer-app @kind logic */
import type { FilterClause } from '@drizztdourden08/tessera';
import type { ClauseField } from '../BoardTabsRow.type';

const picked = (clause: FilterClause | undefined, all: readonly string[]): string[] => {
  const value = Array.isArray(clause?.value) ? (clause.value as unknown[]).map(String) : [];
  if (clause?.op === 'noneOf') return value.length > 0 ? all.filter((option) => !value.includes(option)) : [];
  return clause?.op === 'anyOf' ? value : [];
};

const patchOf = <F>(field: ClauseField<F>, clause: FilterClause | undefined): Partial<F> => {
  if (field.kind === 'flag') return field.write(clause?.op === 'isTrue');
  const values = picked(clause, field.options.map((option) => option.value));
  return field.kind === 'multi' ? field.write(values) : field.write(values.at(-1) ?? '');
};

const fromClauses = <F>(fields: readonly ClauseField<F>[], clauses: readonly FilterClause[]): Partial<F> =>
  fields.reduce<Partial<F>>(
    (patch, field) => ({ ...patch, ...patchOf(field, clauses.find((clause) => clause.enabled && clause.path === field.path)) }),
    {},
  );

export { fromClauses };
