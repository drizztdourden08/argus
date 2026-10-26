/* @layer renderer-app @kind component */
import { EmptyState } from '@drizztdourden08/tessera';
import type { CardSectionsProps } from '../OwnerFeed.type';
import { FeedGroup } from './FeedGroup';

const CardSections = (props: CardSectionsProps) => {
  const { filter, waiting, notices, actions } = props;
  return (
    <>
      {waiting.length > 0 ? <FeedGroup heading="Waiting on you" cards={waiting} actions={actions} /> : null}
      {waiting.length === 0 && filter === 'you' ? <EmptyState message="Nothing waiting on you." size="sm" /> : null}
      {filter === 'all' && notices.length > 0 ? <FeedGroup heading="Notices" cards={notices} actions={actions} /> : null}
    </>
  );
};

export { CardSections };
