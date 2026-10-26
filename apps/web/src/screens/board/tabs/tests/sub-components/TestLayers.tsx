/* @layer renderer-app @kind component */
import { RunLogDrawer } from '../../../run-terminal';
import type { TestLayersProps } from '../tests-drawer-props.type';
import { EditDialog } from './EditDialog';
import { NoteDialog } from './NoteDialog';
import { TestDrawer } from './TestDrawer';

const TestLayers = (props: TestLayersProps) => {
  const { screen, data, boardKey } = props;
  const { visible, anchor, nav, wide, notices } = screen;
  const test = visible.find((candidate) => candidate.id === anchor.selected) ?? null;
  const item = data.items.find((candidate) => candidate.id === test?.item) ?? null;
  const noteTest = visible.find((candidate) => candidate.id === screen.noteId);
  const editTest = visible.find((candidate) => candidate.id === screen.editId);
  return (
    <>
      <TestDrawer test={test} item={item} handed={Boolean(test?.build) && test?.build === data.handed} nav={nav} wide={wide} notices={notices} onClose={anchor.close} />
      {noteTest && <NoteDialog key={noteTest.id} test={noteTest} onClose={screen.closeNote} />}
      {editTest && <EditDialog key={editTest.id} test={editTest} onClose={screen.closeEdit} />}
      <RunLogDrawer run={data.runs.find((run) => run.id === screen.logId) ?? null} boardKey={boardKey} onClose={screen.closeLog} />
    </>
  );
};

export { TestLayers };
