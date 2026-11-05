/* @layer renderer-app @kind logic */
import { formatAgo } from '../../../../../format';
import { RECENT_MAX } from '../board.constants';
import type { BoardPaneData } from '../board.type';

const recentLines = (data: BoardPaneData): string[] => {
  const { card, now } = data;
  const coordinator = card?.coordinator;
  const lines = [
    coordinator?.activity ? `Lead is ${coordinator.activity}.` : '',
    coordinator?.note ?? '',
    card?.master?.updatedAt ? `Master changed ${formatAgo(card.master.updatedAt, now)} ago.` : '',
    ...(card?.attention ?? []),
  ];
  return lines.filter((line) => line !== '').slice(0, RECENT_MAX);
};

export { recentLines };
