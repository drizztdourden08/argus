/* @layer renderer-app @kind component */
import { Card, Flex, Text } from '@drizztdourden08/tessera';
import { RecordHead, useRecordPane } from '../../../../shell';
import { askMode } from '../../feed-model';
import { askLines } from '../behavior/ask-lines';
import { cardFacts } from '../behavior/card-facts';
import { feedHead } from '../behavior/feed-head';
import type { AskCardProps } from '../OwnerFeedAsk.type';
import { AnswerBlock } from './AnswerBlock';
import { ContextPreview } from './ContextPreview';
import { EntryImages } from './EntryImages';

const AskCard = (props: AskCardProps) => {
  const { card, ctx } = props;
  const entry = card.data;
  const pane = useRecordPane();
  const facts = cardFacts(card);
  const lines = askLines(entry.text);
  const title = <Text weight={500} clamp={1}>{lines.title}</Text>;
  return (
    <Card data-feed-type={card.type} data-swap-item edge="left" edgeTone="warning">
      <Flex direction="column" gap="xs" data-record={facts.code}>
        <RecordHead {...feedHead(card, facts, ctx, () => pane.open(facts.code))} title={title} />
        <ContextPreview entry={entry} actions={ctx.actions} onMore={() => pane.open(entry.id)} />
        {lines.rest === '' ? null : <Text smartTimes>{lines.rest}</Text>}
        <EntryImages entry={entry} />
        <AnswerBlock entry={entry} mode={askMode(entry)} actions={ctx.actions} />
      </Flex>
    </Card>
  );
};

export { AskCard };
