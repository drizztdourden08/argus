/* @layer renderer-app @kind logic */
import type { ItemField } from '../../../../../../data/registry';
import type { Option } from '../../../../shared';
import { NO_CHOICE } from './pane-model.constants';

const enumOptions = (spec: ItemField): Option[] => {
  const values = (spec.values ?? []).map((value, index) => ({ value, label: spec.labels?.[index] ? `${value} ${spec.labels[index]}` : value }));
  return spec.required ? values : [{ value: NO_CHOICE, label: 'none' }, ...values];
};

export { enumOptions };
