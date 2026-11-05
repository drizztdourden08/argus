/* @layer renderer-app @kind logic */
import type { FilterClause } from '@drizztdourden08/tessera';
import { FLAG_OPERATOR, LIST_OPERATOR } from '../BoardTabsRow.constants';
import type { TabField } from '../BoardTabsRow.type';

const valueOf = <F>(field: TabField<F>, filters: F): unknown => {
  if (field.kind === 'flag') return null;
  if (field.kind === 'multi') return field.read(filters);
  const single = field.read(filters);
  return single === '' ? [] : [single];
};

const clauseOfField = <F>(field: TabField<F>, filters: F): FilterClause => ({
  id: `field:${field.path}`,
  path: field.path,
  op: field.kind === 'flag' ? FLAG_OPERATOR : LIST_OPERATOR,
  value: valueOf(field, filters),
  enabled: true,
});

export { clauseOfField };
