/* @layer renderer-app @kind logic */
import { isLiveRun } from '../../../../data/runs';
import type { BoardItem } from '../../../../data/boards';
import { attentionOf } from '../../dashboards/model/attention-of';
import { leadState } from '../../dashboards/model/lead-state';
import { EMPTY_BOARD_COUNTS } from './board.constants';
import type { BoardRecord } from './board.type';

const boardRecord = (item: BoardItem, now: number): BoardRecord => {
  const { doc, card } = item;
  return {
    id: doc.id,
    key: doc.key,
    name: doc.name ?? card?.name ?? doc.key,
    icon: doc.icon ?? card?.icon ?? null,
    palette: card?.palette ?? null,
    goal: doc.goal ?? '',
    note: doc.note ?? '',
    rev: doc.rev,
    archived: Boolean(doc.archivedAt),
    started: card !== null && card.state !== 'not started',
    lead: card ? leadState(card, now) : 'not started',
    masterRunning: Boolean(card?.master?.lastRun && isLiveRun(card.master.lastRun)),
    hasMaster: Boolean(card?.master),
    masterMissing: Boolean(card?.masterMissing),
    decisions: card ? attentionOf(card).decisions : 0,
    counts: { ...EMPTY_BOARD_COUNTS, ...card?.counts },
  };
};

export { boardRecord };
