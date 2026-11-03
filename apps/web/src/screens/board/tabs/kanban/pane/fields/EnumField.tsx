/* @layer renderer-app @kind component */
import { fieldOf } from '../../../../../../data/registry';
import { enumOptions } from '../model/enum-options';
import { fieldLabel } from '../model/field-label';
import { fieldText } from '../model/field-text';
import { NO_CHOICE } from '../model/pane-model.constants';
import { saveField } from '../model/save-field';
import type { FieldProps } from '../pane.type';
import { FieldSelect } from './FieldSelect';

const EnumField = (props: FieldProps) => {
  const { name, item } = props;
  const spec = fieldOf(name);
  if (!spec) return null;
  return (
    <FieldSelect
      label={fieldLabel(name)}
      options={enumOptions(spec)}
      value={fieldText(name, item.fields?.[name]) || NO_CHOICE}
      onPick={(next) => saveField(props, next === NO_CHOICE ? null : next)}
    />
  );
};

export { EnumField };
