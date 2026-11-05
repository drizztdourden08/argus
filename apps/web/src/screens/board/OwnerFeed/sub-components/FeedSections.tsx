/* @layer renderer-app @kind component */
import { EmptyState, Flex } from '@drizztdourden08/tessera';
import { isAskCard } from '../behavior/ask-cards';
import type { FeedSectionsProps } from '../OwnerFeed.type';
import { AskGroup } from './AskGroup';
import { FeedGroup } from './FeedGroup';
import { LeadNotes } from './LeadNotes';
import { LogFold } from './LogFold';

const FeedSections = (props: FeedSectionsProps) => {
  const { filter, work, ctx } = props;
  const { waiting, leads, notices, lines } = work;
  if (filter === 'all' && waiting.length + leads.length + notices.length + lines.length === 0) {
    return <EmptyState message="Nothing here yet." size="sm" />;
  }
  const asks = waiting.filter(isAskCard);
  const others = waiting.filter((card) => !isAskCard(card));
  const todos = leads.filter((card) => card.type !== 'sent');
  const sent = leads.filter((card) => card.type === 'sent');
  return (
    <Flex direction="column" gap="md">
      {filter !== 'log' && asks.length > 0 ? <AskGroup heading="Questions and decisions" cards={asks} ctx={ctx} /> : null}
      {filter !== 'log' && others.length > 0 ? <FeedGroup heading="Waiting on you" cards={others} ctx={ctx} /> : null}
      {filter === 'you' && waiting.length === 0 ? <EmptyState message="Nothing waiting on you." size="sm" /> : null}
      {filter === 'all' && leads.length > 0 ? (
        <FeedGroup heading="For the leads" cards={todos} total={leads.length} ctx={ctx} extra={<LeadNotes cards={sent} ctx={ctx} />} />
      ) : null}
      {filter === 'you' ? null : <LogFold filter={filter} notices={notices} lines={lines} ctx={ctx} />}
    </Flex>
  );
};

export { FeedSections };
