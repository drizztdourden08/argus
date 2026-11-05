/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { RecordRow, useRecordPane } from '../../../../shell';
import { formatAgo } from '../../../../format';
import { cardFacts } from '../behavior/card-facts';
import { feedHead } from '../behavior/feed-head';
import type { EntryRowProps } from '../OwnerFeed.type';

const EntryRow = (props: EntryRowProps) => {
  const { type, entry, ctx, trailing } = props;
  const pane = useRecordPane();
  const card = { type, at: entry.createdAt, data: entry } as const;
  const facts = cardFacts(card);
  const open = () => pane.open(facts.code);
  return (
    <Flex direction="column" data-swap-item>
      <RecordRow head={feedHead(card, facts, ctx, open)} title={facts.title} onOpen={open} age={formatAgo(entry.createdAt, ctx.now)} trailing={trailing} />
    </Flex>
  );
};

export { EntryRow };
