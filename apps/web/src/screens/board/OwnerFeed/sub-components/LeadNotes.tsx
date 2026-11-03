/* @layer renderer-app @kind component */
import { Disclosure, Flex } from '@drizztdourden08/tessera';
import { leadNotes } from '../behavior/lead-notes';
import type { LeadNotesProps } from '../OwnerFeed.type';
import { SentRow } from './SentRow';

const LeadNotes = (props: LeadNotesProps) => {
  const { cards, ctx } = props;
  return (
    <Flex direction="column" gap="xs">
      {leadNotes(cards).map((group) => (
        <Disclosure key={group.lead} summary={`${group.lead} · ${group.entries.length} not read yet`} size="sm">
          <Flex direction="column" gap="2xs" role="group" aria-label={`Sent to ${group.lead}`}>
            {group.entries.map((entry) => <SentRow key={entry.id} entry={entry} ctx={ctx} />)}
          </Flex>
        </Disclosure>
      ))}
    </Flex>
  );
};

export { LeadNotes };
