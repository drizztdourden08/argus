/* @layer renderer-app @kind component */
import { fieldOf } from '../../../../data/registry';
import { EditableMarkdown, useBoardKey, useItemMedia } from '../../../../screens/board/shared';
import { useRecordPane } from '../../behavior/useRecordPane';
import { fieldLabel } from '../model/field-label';
import { fieldProblem } from '../model/field-problem';
import { fieldText } from '../model/field-text';
import { fieldVerdict } from '../model/field-verdict';
import { saveField } from '../model/save-field';
import type { FieldProps } from './fields.type';

const RichField = (props: FieldProps) => {
  const { name, item, cx } = props;
  const noun = fieldLabel(name).toLowerCase();
  const routed = useBoardKey();
  const opened = useRecordPane().ref?.board;
  const board = routed ?? opened ?? undefined;
  const { images, media } = useItemMedia(board, item);
  const known = { ...cx, images: images.map((image) => image.n) };
  const save = (text: string) => {
    const verdict = fieldVerdict(name, text, known);
    if (verdict.ok) saveField(props, verdict.value);
  };
  return (
    <EditableMarkdown
      noun={noun}
      value={fieldText(name, item.fields?.[name])}
      empty={`add ${noun}`}
      limit={fieldOf(name)?.max ?? 0}
      problem={(text) => fieldProblem(name, text, known) ?? null}
      media={media}
      onSave={save}
    />
  );
};

export { RichField };
