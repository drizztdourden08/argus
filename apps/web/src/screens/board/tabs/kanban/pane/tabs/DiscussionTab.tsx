/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { EntrySection } from '../../sub-components/EntrySection';
import { TodosSection } from '../../sub-components/TodosSection';
import type { PaneBodyProps } from '../pane.type';

const DiscussionTab = (props: PaneBodyProps) => {
  const { item, record, feed, ops, acts, now } = props.data;
  const openQuestions = record.questions.filter((row) => row.open).length;
  return (
    <Flex direction="column" gap="lg">
      <EntrySection title={`Questions · ${openQuestions} open`} empty="No question on this item." rows={record.questions} feed={feed} ops={ops} now={now} />
      <EntrySection title="Decisions" empty="No decision recorded yet." rows={record.decisions} feed={feed} ops={ops} now={now} />
      <TodosSection item={item} todos={record.todos} ops={ops} acts={acts} />
      <EntrySection title="Notes" empty="No note on this item." rows={record.notes} feed={feed} ops={ops} now={now} />
    </Flex>
  );
};

export { DiscussionTab };
