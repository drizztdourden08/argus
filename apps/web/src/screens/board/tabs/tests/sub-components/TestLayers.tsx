/* @layer renderer-app @kind component */
import { RunLogDrawer } from '../../../run-terminal';
import type { TestLayersProps } from '../tests-drawer-props.type';
import { EditDialog } from './EditDialog';
import { NoteDialog } from './NoteDialog';

const TestLayers = (props: TestLayersProps) => {
  const { screen, data, boardKey } = props;
  const noteTest = screen.visible.find((candidate) => candidate.id === screen.noteId);
  const editTest = screen.visible.find((candidate) => candidate.id === screen.editId);
  return (
    <>
      {noteTest && <NoteDialog key={noteTest.id} test={noteTest} onClose={screen.closeNote} />}
      {editTest && <EditDialog key={editTest.id} test={editTest} onClose={screen.closeEdit} />}
      <RunLogDrawer run={data.runs.find((run) => run.id === screen.logId) ?? null} boardKey={boardKey} onClose={screen.closeLog} />
    </>
  );
};

export { TestLayers };
