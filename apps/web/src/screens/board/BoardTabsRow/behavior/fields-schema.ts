/* @layer renderer-app @kind logic */
import type { FieldDescriptor } from '@drizztdourden08/tessera';
import { FLAG_OPERATOR, LIST_OPERATOR } from '../BoardTabsRow.constants';
import type { TabField } from '../BoardTabsRow.type';

const descriptorOf = <F>(field: TabField<F>): FieldDescriptor => {
  if (field.kind === 'flag') {
    return { path: field.path, label: field.label, kind: 'boolean', optional: true, operators: [FLAG_OPERATOR] };
  }
  return {
    path: field.path,
    label: field.label,
    kind: 'enum',
    optional: true,
    closed: true,
    options: field.options.map((option) => option.value),
    declaredOptions: field.options,
    operators: [LIST_OPERATOR],
  };
};

const fieldsSchema = <F>(fields: readonly TabField<F>[]): readonly FieldDescriptor[] => fields.map(descriptorOf);

export { fieldsSchema };
