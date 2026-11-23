/* @layer renderer-app @kind component */
import { Text, Tooltip } from '@drizztdourden08/tessera';
import { Age, RecordHead, useRecordPane } from '../../../../shell';
import { askMode } from '../../feed-model';
import { askLines } from '../behavior/ask-lines';
import { cardFacts } from '../behavior/card-facts';
import { feedHead } from '../behavior/feed-head';
import type { AskCardProps } from '../behavior/ask.type';
import { AnswerBlock } from './AnswerBlock';
import { ContextPreview } from './ContextPreview';
import { EntryImages } from './EntryImages';

const AskCard = (props: AskCardProps) => {
  const { card, ctx } = props;
  const entry = card.data;
  const pane = useRecordPane();
  const facts = cardFacts(card);
  const lines = askLines(entry.text);
  const title = (
    <Tooltip content={lines.title}>
      <Text weight={500} clamp={1}>{lines.title}</Text>
    </Tooltip>
  );
  const time = <Text as="span" variant="caption" tone="muted"><Age value={entry.createdAt} /></Text>;
  return (
    <RecordHead
      {...feedHead(facts, ctx, () => pane.open(facts.code))}
      title={title}
      content={time}
      density="row"
      defaultOpen
      edge="left"
      edgeTone="warning"
      attrs={{ role: 'listitem', 'data-feed-type': card.type, 'data-swap-item': '', 'data-record': facts.code }}
    >
      <ContextPreview entry={entry} actions={ctx.actions} onMore={() => pane.open(entry.id)} />
      {lines.rest === '' ? null : <Text smartTimes>{lines.rest}</Text>}
      <EntryImages entry={entry} />
      <AnswerBlock entry={entry} mode={askMode(entry)} actions={ctx.actions} />
    </RecordHead>
  );
};

export { AskCard };
