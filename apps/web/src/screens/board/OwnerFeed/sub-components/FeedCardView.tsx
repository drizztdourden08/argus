/* @layer renderer-app @kind component */
import { Card } from '@drizztdourden08/tessera';
import { RecordCard, useRecordPane } from '../../../../shell';
import { sectionOf } from '../../feed-model';
import { cardFacts } from '../behavior/card-facts';
import { feedMenu } from '../behavior/feed-menu';
import type { FeedCardViewProps } from '../OwnerFeed.type';
import { CardBody } from './CardBody';
import { FeedFacts } from './FeedFacts';

const FeedCardView = (props: FeedCardViewProps) => {
  const { card, ctx } = props;
  const pane = useRecordPane();
  const facts = cardFacts(card);
  const open = () => pane.open(facts.code, facts.tab);
  return (
    <Card data-feed-type={card.type} edge="left" edgeTone={sectionOf(card) === 'waiting' ? 'warning' : undefined}>
      <RecordCard
        head={{
          kind: facts.kind,
          code: facts.code,
          assignee: facts.assignee,
          boardKey: ctx.boardKey,
          threads: ctx.threads,
          menu: feedMenu(card, facts, ctx.actions, open),
          menuLabel: `More actions for ${facts.code}`,
        }}
        title={facts.title}
        openLabel={`Open ${facts.code}: ${facts.title}`}
        onOpen={open}
        signals={<FeedFacts facts={facts} index={ctx.index} />}
        foot={<CardBody card={card} ctx={ctx} facts={facts} />}
        attrs={{ 'data-record': facts.code }}
      />
    </Card>
  );
};

export { FeedCardView };
