/* @layer renderer-app @kind component */
import { Card, Flex } from '@drizztdourden08/tessera';
import { FEED_META } from '../../feed-model';
import type { FeedCardViewProps } from '../OwnerFeed.type';
import { CardBody } from './CardBody';
import { CardMenu } from './CardMenu';
import { CardMeta } from './CardMeta';
import { TestLine } from './TestLine';

const FeedCardView = (props: FeedCardViewProps) => {
  const { card, actions } = props;
  if (card.type === 'test' || card.type === 'bundle') {
    return (
      <Card edgeTone="warning" data-feed-type={card.type}>
        <TestLine card={card} handed={actions.handed} />
      </Card>
    );
  }
  return (
    <Card data-feed-type={card.type} edge="left" edgeTone={FEED_META[card.type].you ? 'warning' : undefined}>
      <Flex direction="column" gap="xs">
        <CardMeta
          label={FEED_META[card.type].label}
          id={card.data.id}
          at={card.at}
          thread={card.data.thread}
          item={card.data.item}
          menu={<CardMenu collection={card.type === 'action' ? 'actions' : 'entries'} id={card.data.id} actions={actions} />}
        />
        <CardBody card={card} actions={actions} />
      </Flex>
    </Card>
  );
};

export { FeedCardView };
