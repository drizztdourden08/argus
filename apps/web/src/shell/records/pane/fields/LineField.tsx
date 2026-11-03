/* @layer renderer-app @kind component */
import { InlineEdit } from '@drizztdourden08/tessera';
import { fieldOf } from '../../../../data/registry';
import { fieldLabel } from '../model/field-label';
import { fieldProblem } from '../model/field-problem';
import { fieldText } from '../model/field-text';
import { fieldVerdict } from '../model/field-verdict';
import { saveField } from '../model/save-field';
import type { FieldProps } from './fields.type';

const LineField = (props: FieldProps) => {
  const { name, item, cx } = props;
  const label = fieldLabel(name);
  const value = fieldText(name, item.fields?.[name]);
  const commit = (text: string) => {
    const verdict = fieldVerdict(name, text, cx);
    if (verdict.ok) saveField(props, verdict.value);
  };
  return (
    <InlineEdit
      label={label}
      placeholder={`add ${label.toLowerCase()}`}
      multiline={fieldOf(name)?.list !== undefined}
      value={value}
      validate={(text) => fieldProblem(name, text, cx)}
      onCommit={commit}
    />
  );
};

export { LineField };
