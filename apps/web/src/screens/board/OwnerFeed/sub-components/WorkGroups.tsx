/* @layer renderer-app @kind component */
import { EmptyState } from '@drizztdourden08/tessera';
import { splitWork } from '../behavior/split-work';
import type { FeedSectionsProps } from '../OwnerFeed.type';
import { AskGroup } from './AskGroup';
import { FeedGroup } from './FeedGroup';
import { LeadNotes } from './LeadNotes';

const WorkGroups = ({ filter, work, ctx }: FeedSectionsProps) => {
  const { waiting, leads } = work;
  const { asks, others, todos, sent } = splitWork(work);
  return (
    <>
      {filter !== 'log' && asks.length > 0 ? <AskGroup heading="Questions and decisions" cards={asks} ctx={ctx} /> : null}
      {filter !== 'log' && others.length > 0 ? <FeedGroup heading="Waiting on you" cards={others} ctx={ctx} /> : null}
      {filter === 'you' && waiting.length === 0 ? <EmptyState message="Nothing waiting on you." size="sm" /> : null}
      {filter === 'all' && leads.length > 0 ? (
        <FeedGroup heading="For the leads" cards={todos} total={leads.length} ctx={ctx} extra={<LeadNotes cards={sent} ctx={ctx} />} />
      ) : null}
    </>
  );
};

export { WorkGroups };
