/* @layer renderer-app @kind component */
import { fieldOf } from '../../../../../../data/registry';
import { EditableMarkdown } from '../../../../shared';
import { fieldLabel } from '../model/field-label';
import { fieldProblem } from '../model/field-problem';
import { fieldText } from '../model/field-text';
import { fieldVerdict } from '../model/field-verdict';
import { saveField } from '../model/save-field';
import type { FieldProps } from '../pane.type';

const RichField = (props: FieldProps) => {
  const { name, item, cx } = props;
  const noun = fieldLabel(name).toLowerCase();
  const save = (text: string) => {
    const verdict = fieldVerdict(name, text, cx);
    if (verdict.ok) saveField(props, verdict.value);
  };
  return (
    <EditableMarkdown
      noun={noun}
      value={fieldText(name, item.fields?.[name])}
      empty={`add ${noun}`}
      limit={fieldOf(name)?.max ?? 0}
      problem={(text) => fieldProblem(name, text, cx) ?? null}
      onSave={save}
    />
  );
};

export { RichField };
