/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { EntrySection } from '../../../../screens/board/tabs/kanban/sub-components/EntrySection';
import type { RecordBodyProps } from '../pane.type';

const RecordDiscussionTab = (props: RecordBodyProps) => {
  const { code, record, feed, ops, now } = props.data;
  const openQuestions = record.questions.filter((row) => row.open).length;
  return (
    <Flex direction="column" gap="lg">
      <EntrySection title={`Questions · ${openQuestions} open`} empty={`No question on ${code}.`} rows={record.questions} feed={feed} ops={ops} now={now} />
      <EntrySection title="Decisions" empty="No decision recorded yet." rows={record.decisions} feed={feed} ops={ops} now={now} />
      <EntrySection title="Notes" empty={`No note on ${code}.`} rows={record.notes} feed={feed} ops={ops} now={now} />
    </Flex>
  );
};

export { RecordDiscussionTab };
