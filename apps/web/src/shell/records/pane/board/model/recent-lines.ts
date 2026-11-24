/* @layer renderer-app @kind logic */
import { formatAgo } from '../../../../../format';
import { RECENT_MAX } from '../board.constants';
import type { BoardPaneData } from '../board.type';

const leadLines = (card: BoardPaneData['card']): string[] => {
  const lead = card?.lead;
  return [lead?.activity ? `Lead is ${lead.activity}.` : '', lead?.note ?? ''];
};

const recentLines = (data: BoardPaneData): string[] => {
  const { card, now } = data;
  const lines = [
    ...leadLines(card),
    card?.master?.updatedAt ? `Master changed ${formatAgo(card.master.updatedAt, now)} ago.` : '',
    ...(card?.attention ?? []),
  ];
  return lines.filter((line) => line !== '').slice(0, RECENT_MAX);
};

export { recentLines };
