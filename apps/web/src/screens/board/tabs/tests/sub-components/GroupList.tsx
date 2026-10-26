/* @layer renderer-app @kind component */
import { EmptyState, Flex } from '@drizztdourden08/tessera';
import { NO_MATCH_TEXT, NO_TESTS_TEXT } from '../tests.constants';
import type { GroupListProps } from '../tests-props.type';
import { TestCard } from './TestCard';

const GroupList = (props: GroupListProps) => {
  const { cards, filtered } = props;
  if (cards.length === 0) return <EmptyState message={filtered ? NO_MATCH_TEXT : NO_TESTS_TEXT} />;
  return (
    <Flex direction="column" gap="md">
      {cards.map((card) => <TestCard key={card.key} card={card} />)}
    </Flex>
  );
};

export { GroupList };
