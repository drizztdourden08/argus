/* @layer renderer-app @kind component */
import { RecordRow, useRecordPane } from '../../../../shell';
import { formatAgo } from '../../../../format';
import { cardFacts } from '../behavior/card-facts';
import { feedHead } from '../behavior/feed-head';
import type { SentRowProps } from '../OwnerFeed.type';

const SentRow = (props: SentRowProps) => {
  const { entry, ctx } = props;
  const pane = useRecordPane();
  const card = { type: 'sent', at: entry.createdAt, data: entry } as const;
  const facts = cardFacts(card);
  const open = () => pane.open(facts.code);
  return <RecordRow head={feedHead(card, facts, ctx, open)} title={facts.title} onOpen={open} age={formatAgo(entry.createdAt, ctx.now)} />;
};

export { SentRow };
