/* @layer renderer-app @kind component */
import { Flex, Grid, Text } from '@drizztdourden08/tessera';
import { FEED_CARD_MIN_WIDTH } from '../OwnerFeed.constants';
import type { FeedGroupProps } from '../OwnerFeed.type';
import { FeedCardView } from './FeedCardView';

const FeedGroup = (props: FeedGroupProps) => {
  const { heading, cards, ctx, total = cards.length, extra } = props;
  return (
    <Flex direction="column" gap="sm" role="group" aria-label={heading}>
      <Text variant="overline" tone="muted">{heading} · {total}</Text>
      <Grid minColWidth={FEED_CARD_MIN_WIDTH} gap="md">
        {cards.map((card) => (
          <FeedCardView key={`${card.type}:${card.data.id}`} card={card} ctx={ctx} />
        ))}
      </Grid>
      {extra}
    </Flex>
  );
};

export { FeedGroup };
