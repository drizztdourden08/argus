/* @layer renderer-app @kind logic */
import type { FieldDescriptor, FilterClause } from '@drizztdourden08/tessera';
import { CHOICE_OPERATORS, FLAG_OPERATORS } from '../BoardTabsRow.constants';
import type { ClauseField } from '../BoardTabsRow.type';

const descriptorOf = <F>(field: ClauseField<F>): FieldDescriptor => {
  if (field.kind === 'flag') return { path: field.path, label: field.label, kind: 'boolean', optional: true, operators: FLAG_OPERATORS };
  return {
    path: field.path,
    label: field.label,
    kind: 'enum',
    optional: true,
    closed: true,
    operators: CHOICE_OPERATORS,
    options: field.options.map((option) => option.value),
    declaredOptions: field.options,
  };
};

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
  Object.assign({}, ...fields.map((field) => patchOf(field, clauses.find((clause) => clause.enabled && clause.path === field.path))));

const lastOnly = (clause: FilterClause): FilterClause =>
  Array.isArray(clause.value) && clause.value.length > 1 ? { ...clause, value: clause.value.slice(-1) } : clause;

const settled = <F>(fields: readonly ClauseField<F>[], clauses: readonly FilterClause[]): FilterClause[] =>
  clauses.map((clause) => (fields.some((field) => field.kind === 'single' && field.path === clause.path) ? lastOnly(clause) : clause));

export { descriptorOf, fromClauses, settled, toClauses };
