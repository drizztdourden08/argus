/* @layer renderer-app @kind logic */
import { formatAgo } from '../../../../../format';
import type { BoardFact, BoardPaneData } from '../board.type';

const ago = (iso: string | null | undefined, now: number): string => (iso ? `${formatAgo(iso, now)} ago` : 'never');

const boardFacts = (data: BoardPaneData): BoardFact[] => {
  const { record, card, now } = data;
  return [
    { label: 'Lead', text: record.lead },
    { label: 'Seen', text: ago(card?.lead?.lastSeen, now) },
    { label: 'Root', text: card?.root ?? 'none' },
    { label: 'State', text: record.archived ? 'archived' : (card?.state ?? 'not started') },
    { label: 'Palette', text: record.palette ?? 'basic' },
    { label: 'Updated', text: ago(card?.updatedAt, now) },
  ];
};

export { boardFacts };
