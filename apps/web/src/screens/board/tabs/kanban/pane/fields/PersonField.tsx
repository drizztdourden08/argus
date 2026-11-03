/* @layer renderer-app @kind component */
import { fieldOf } from '../../../../../../data/registry';
import { fieldLabel } from '../model/field-label';
import { fieldText } from '../model/field-text';
import { fieldVerdict } from '../model/field-verdict';
import { NO_CHOICE } from '../model/pane-model.constants';
import { saveField } from '../model/save-field';
import type { FieldProps } from '../pane.type';
import { FieldSelect } from './FieldSelect';

const PersonField = (props: FieldProps) => {
  const { name, item, people, cx } = props;
  const current = fieldText(name, item.fields?.[name]);
  const known = current === '' || people.some((person) => person.value === current);
  const none = fieldOf(name)?.required ? [] : [{ value: NO_CHOICE, label: 'none' }];
  const options = [...none, ...people, ...(known ? [] : [{ value: current, label: current }])];
  const pick = (next: string) => {
    const verdict = fieldVerdict(name, next, cx);
    if (next === NO_CHOICE) saveField(props, null);
    else if (verdict.ok) saveField(props, verdict.value);
  };
  return <FieldSelect label={fieldLabel(name)} options={options} value={current || NO_CHOICE} onPick={pick} />;
};

export { PersonField };
