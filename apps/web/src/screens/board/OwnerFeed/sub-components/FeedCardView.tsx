/* @layer renderer-app @kind component */
import { Card, Flex } from '@drizztdourden08/tessera';
import { FEED_META } from '../../feed-model';
import { CARD_TONES } from '../OwnerFeed.constants';
import type { FeedCardViewProps } from '../OwnerFeed.type';
import { CardBody } from './CardBody';
import { CardMeta } from './CardMeta';

const FeedCardView = (props: FeedCardViewProps) => {
  const { card, actions } = props;
  const item = card.type === 'bundle' ? null : card.data.item;
  return (
    <Card title={FEED_META[card.type].label} tone={CARD_TONES[card.type]} level={3} data-feed-type={card.type}>
      <Flex direction="column" gap="sm">
        <CardMeta id={card.data.id} at={card.at} thread={card.data.thread} item={item} />
        <CardBody card={card} actions={actions} />
      </Flex>
    </Card>
  );
};

export { FeedCardView };
