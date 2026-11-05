/* @layer renderer-app @kind component */
import { EditableMarkdown } from '../../../../screens/board/shared';
import { fieldOf } from '../../../../data/registry';
import { useBoardWrite } from './behavior/useBoardWrite';
import { EDIT_DONE, EDIT_EMPTY } from './board.constants';
import type { BoardTextEditorProps } from './board.type';
import { boardEditOp } from './model/board-edit-op';
import { boardTextProblem } from './model/board-text-problem';

const BoardTextEditor = (props: BoardTextEditorProps) => {
  const { record, field } = props;
  const write = useBoardWrite();
  return (
    <EditableMarkdown
      noun={field}
      value={record[field]}
      empty={EDIT_EMPTY[field] ?? ''}
      limit={fieldOf(field)?.max ?? 0}
      problem={(text) => boardTextProblem(field, text)}
      onSave={(value) => void write(boardEditOp(record, field, value), EDIT_DONE[field] ?? '')}
    />
  );
};

export { BoardTextEditor };
