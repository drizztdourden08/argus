/* @layer renderer-app @kind logic */
import type { FilterClause } from '@drizztdourden08/tessera';
import type { TabField } from '../BoardTabsRow.type';
import { fieldIsSet } from './field-is-set';
import { resetField } from './reset-field';

const listOf = (clause: FilterClause): string[] => (Array.isArray(clause.value) ? clause.value.map(String) : []);

const newest = (values: readonly string[], current: string): string => {
  if (values.length === 0) return '';
  return values.filter((value) => value !== current).at(-1) ?? current;
};

const writeClause = <F>(field: TabField<F>, clause: FilterClause, filters: F): Partial<F> => {
  if (field.kind === 'flag') return field.write(true);
  if (field.kind === 'multi') return field.write(listOf(clause));
  return field.write(newest(listOf(clause), field.read(filters)));
};

const fieldPatch = <F>(field: TabField<F>, clauses: readonly FilterClause[], filters: F): Partial<F> => {
  const clause = clauses.find((candidate) => candidate.path === field.path);
  if (clause) return writeClause(field, clause, filters);
  return fieldIsSet(field, filters) ? resetField(field) : {};
};

const clausePatch = <F>(fields: readonly TabField<F>[], clauses: readonly FilterClause[], filters: F): Partial<F> =>
  fields.reduce<Partial<F>>((patch, field) => ({ ...patch, ...fieldPatch(field, clauses, filters) }), {});

export { clausePatch };
