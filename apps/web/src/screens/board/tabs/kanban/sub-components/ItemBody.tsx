/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { useItemRecord } from '../behavior/useItemRecord';
import { useRecordActions } from '../behavior/useRecordActions';
import type { ItemBodyProps } from '../drawer-props.type';
import { useFeedActions } from '../../../OwnerFeed';
import { ActivitySection } from './ActivitySection';
import { CriteriaSection } from './CriteriaSection';
import { DescriptionSection } from './DescriptionSection';
import { DrawerFacts } from './DrawerFacts';
import { EntrySection } from './EntrySection';
import { ItemTests } from './ItemTests';
import { TodosSection } from './TodosSection';

const ItemBody = (props: ItemBodyProps) => {
  const { boardKey, item, tests, threads, actions, waiting, now, ops } = props;
  const record = useItemRecord(boardKey, item.id, actions);
  const acts = useRecordActions(boardKey);
  const feed = useFeedActions(boardKey);
  const own = tests.filter((test) => test.item === item.id && !test.deletedAt);
  const openQuestions = record.questions.filter((row) => row.open).length;
  return (
    <Flex direction="column" gap="lg">
      <DrawerFacts item={item} waiting={waiting} threads={threads} now={now} ops={ops} />
      <DescriptionSection item={item} ops={ops} />
      <CriteriaSection item={item} tests={own} ops={ops} />
      <TodosSection item={item} todos={record.todos} ops={ops} acts={acts} />
      <ItemTests tests={own} />
      <EntrySection title={`Questions · ${openQuestions} open`} empty="No question on this item." rows={record.questions} feed={feed} ops={ops} now={now} />
      <EntrySection title="Decisions" empty="No decision recorded yet." rows={record.decisions} feed={feed} ops={ops} now={now} />
      <ActivitySection item={item} rows={record.timeline} acts={acts} now={now} />
    </Flex>
  );
};

export { ItemBody };
