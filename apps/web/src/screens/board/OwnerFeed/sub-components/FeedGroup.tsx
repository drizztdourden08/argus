/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { FeedGroupProps } from '../OwnerFeed.type';
import { FeedCardView } from './FeedCardView';

const FeedGroup = (props: FeedGroupProps) => {
  const { heading, cards, ctx, total = cards.length, extra } = props;
  return (
    <Flex direction="column" gap="sm" role="group" aria-label={heading}>
      <Text variant="overline" tone="muted">{heading} · {total}</Text>
      {cards.map((card) => (
        <FeedCardView key={`${card.type}:${card.data.id}`} card={card} ctx={ctx} />
      ))}
      {extra}
    </Flex>
  );
};

export { FeedGroup };
