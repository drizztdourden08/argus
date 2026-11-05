/* @layer renderer-app @kind logic */
import { isLiveRun } from '../../../../data/runs';
import type { BoardItem } from '../../../../data/boards';
import { attentionOf } from '../../dashboards/model/attention-of';
import { leadState } from '../../dashboards/model/lead-state';
import { EMPTY_BOARD_COUNTS } from './board.constants';
import type { BoardItemCard, BoardItemDoc, BoardRecord } from './board.type';

const look = (doc: BoardItemDoc, card: BoardItemCard): Pick<BoardRecord, 'name' | 'icon' | 'palette'> => ({
  name: doc.name ?? card?.name ?? doc.key,
  icon: doc.icon ?? card?.icon ?? null,
  palette: card?.palette ?? null,
});

const identity = (doc: BoardItemDoc, card: BoardItemCard): Pick<BoardRecord, 'id' | 'key' | 'name' | 'icon' | 'palette' | 'goal' | 'note' | 'rev'> => ({
  id: doc.id,
  key: doc.key,
  ...look(doc, card),
  goal: doc.goal ?? '',
  note: doc.note ?? '',
  rev: doc.rev,
});

const master = (card: BoardItemCard): Pick<BoardRecord, 'masterRunning' | 'hasMaster' | 'masterMissing'> => ({
  masterRunning: Boolean(card?.master?.lastRun && isLiveRun(card.master.lastRun)),
  hasMaster: Boolean(card?.master),
  masterMissing: Boolean(card?.masterMissing),
});

const standing = (doc: BoardItemDoc, card: BoardItemCard, now: number): Pick<BoardRecord, 'archived' | 'started' | 'lead' | 'decisions' | 'counts'> => ({
  archived: Boolean(doc.archivedAt),
  started: card !== null && card.state !== 'not started',
  lead: card ? leadState(card, now) : 'not started',
  decisions: card ? attentionOf(card).decisions : 0,
  counts: { ...EMPTY_BOARD_COUNTS, ...card?.counts },
});

const boardRecord = (item: BoardItem, now: number): BoardRecord => {
  const { doc, card } = item;
  return { ...identity(doc, card), ...master(card), ...standing(doc, card, now) };
};

export { boardRecord };
