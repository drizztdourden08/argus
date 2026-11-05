/* @layer renderer-app @kind logic */
import type { FieldDescriptor } from '@drizztdourden08/tessera';
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

export { descriptorOf };
