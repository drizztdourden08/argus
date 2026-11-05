/* @layer renderer-app @kind component */
import { RecordCard, useRecordPane } from '../../../../shell';
import { sectionOf } from '../../feed-model';
import { cardFacts } from '../behavior/card-facts';
import { feedHead } from '../behavior/feed-head';
import type { FeedCardViewProps } from '../OwnerFeed.type';
import { CardBody } from './CardBody';
import { FeedFacts } from './FeedFacts';

const FeedCardView = (props: FeedCardViewProps) => {
  const { card, ctx } = props;
  const pane = useRecordPane();
  const facts = cardFacts(card);
  const open = () => pane.open(facts.code, facts.tab);
  return (
    <RecordCard
      head={feedHead(card, facts, ctx, open)}
      title={facts.title}
      openLabel={`Open ${facts.code}: ${facts.title}`}
      onOpen={open}
      label={`${facts.code}: ${facts.title}`}
      edge="left"
      edgeTone={sectionOf(card) === 'waiting' ? 'warning' : undefined}
      signals={<FeedFacts facts={facts} index={ctx.index} />}
      foot={<CardBody card={card} ctx={ctx} />}
      attrs={{ 'data-feed-type': card.type, 'data-swap-item': '', 'data-record': facts.code }}
    />
  );
};

export { FeedCardView };
