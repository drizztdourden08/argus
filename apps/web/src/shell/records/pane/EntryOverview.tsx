/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { EntryBlock } from '../../../screens/board/tabs/kanban/sub-components/EntryBlock';
import { EntryFacts } from './EntryFacts';
import type { RecordBodyProps } from './pane.type';

const EntryOverview = (props: RecordBodyProps) => {
  const { entry, feed, ops, now, open } = props.data;
  if (!entry) return null;
  return (
    <Flex direction="column" gap="md">
      <EntryFacts entry={entry.entry} onOpenItem={open} />
      <EntryBlock row={entry} feed={feed} ops={ops} now={now} />
    </Flex>
  );
};

export { EntryOverview };
