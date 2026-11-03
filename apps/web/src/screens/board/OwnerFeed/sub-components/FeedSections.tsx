/* @layer renderer-app @kind component */
import { EmptyState, Flex } from '@drizztdourden08/tessera';
import type { FeedSectionsProps } from '../OwnerFeed.type';
import { FeedGroup } from './FeedGroup';
import { LogFold } from './LogFold';

const FeedSections = (props: FeedSectionsProps) => {
  const { filter, work, ctx } = props;
  const { waiting, leads, notices, lines } = work;
  if (filter === 'all' && waiting.length + leads.length + notices.length + lines.length === 0) {
    return <EmptyState message="Nothing here yet." size="sm" />;
  }
  return (
    <Flex direction="column" gap="md">
      {filter !== 'log' && waiting.length > 0 ? <FeedGroup heading="Waiting on you" cards={waiting} ctx={ctx} /> : null}
      {filter === 'you' && waiting.length === 0 ? <EmptyState message="Nothing waiting on you." size="sm" /> : null}
      {filter === 'all' && leads.length > 0 ? <FeedGroup heading="For the leads" cards={leads} ctx={ctx} /> : null}
      {filter === 'you' ? null : <LogFold filter={filter} notices={notices} lines={lines} ctx={ctx} />}
    </Flex>
  );
};

export { FeedSections };
