/* @layer renderer-app @kind logic */
import type { Entry } from '../../../../data/types';
import type { FeedCard } from '../../feed-model';
import { NO_LEAD } from '../OwnerFeed.constants';
import type { LeadNotesGroup } from '../OwnerFeed.type';

const leadNotes = (cards: readonly FeedCard[]): LeadNotesGroup[] => {
  const groups = new Map<string, Entry[]>();
  for (const card of cards) {
    if (card.type !== 'sent') continue;
    const lead = card.data.thread ?? NO_LEAD;
    groups.set(lead, [...(groups.get(lead) ?? []), card.data]);
  }
  return [...groups].map(([lead, entries]) => ({ lead, entries }));
};

export { leadNotes };
