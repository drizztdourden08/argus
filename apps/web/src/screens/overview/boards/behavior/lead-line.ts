/* @layer renderer-app @kind logic */
import { formatAgo } from '../../../../format';
import type { Card } from '../../../../data/types';
import { LEAD_WORDS } from '../boards.constants';
import type { LeadWords } from '../boards.constants';
import type { BoardLive, BoardRecord } from '../model';

const leadLine = (record: BoardRecord, live: BoardLive, card: Card | null, now: number): LeadWords => {
  if (live.fault) return { text: live.fault.reason === 'board-closed' ? 'Board closed' : 'Wake failed', tone: 'danger' };
  if (record.lead === 'stale') return { text: `Lead not seen for ${formatAgo(card?.coordinator?.lastSeen, now)}`, tone: 'warning' };
  return LEAD_WORDS[record.lead];
};

export { leadLine };
