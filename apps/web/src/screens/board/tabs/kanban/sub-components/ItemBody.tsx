/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { ItemBodyProps } from '../kanban-props.type';
import { CriteriaSection } from './CriteriaSection';
import { DescriptionSection } from './DescriptionSection';
import { DrawerFacts } from './DrawerFacts';
import { ItemTests } from './ItemTests';

const ItemBody = (props: ItemBodyProps) => {
  const { item, tests, threads, now, ops } = props;
  const own = tests.filter((test) => test.item === item.id && !test.deletedAt);
  return (
    <Flex direction="column" gap="lg">
      <DrawerFacts item={item} threads={threads} now={now} ops={ops} />
      <DescriptionSection item={item} ops={ops} />
      <CriteriaSection item={item} tests={own} ops={ops} />
      <ItemTests tests={own} />
    </Flex>
  );
};

export { ItemBody };
