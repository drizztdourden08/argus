/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { FeedGroupProps } from '../OwnerFeed.type';
import { FeedCardView } from './FeedCardView';

const FeedGroup = (props: FeedGroupProps) => {
  const { heading, cards, ctx } = props;
  return (
    <Flex direction="column" gap="sm" role="group" aria-label={heading}>
      <Text variant="overline" tone="muted">{heading} · {cards.length}</Text>
      {cards.map((card) => (
        <FeedCardView key={`${card.type}:${card.data.id}`} card={card} ctx={ctx} />
      ))}
    </Flex>
  );
};

export { FeedGroup };
