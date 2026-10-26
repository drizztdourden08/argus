/* @layer renderer-app @kind component */
import { EmptyState, Flex } from '@drizztdourden08/tessera';
import type { FeedSectionsProps } from '../OwnerFeed.type';
import { CardSections } from './CardSections';
import { LogSection } from './LogSection';

const FeedSections = (props: FeedSectionsProps) => {
  const { filter, work, actions } = props;
  const { waiting, notices, lines } = work;
  if (filter === 'all' && waiting.length + notices.length + lines.length === 0) {
    return <EmptyState message="Nothing here yet." size="sm" />;
  }
  return (
    <Flex direction="column" gap="md">
      {filter === 'log' ? null : <CardSections filter={filter} waiting={waiting} notices={notices} actions={actions} />}
      {filter === 'you' ? null : <LogSection lines={lines} />}
    </Flex>
  );
};

export { FeedSections };
