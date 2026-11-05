/* @layer renderer-app @kind logic */
import { attentionGroup } from './attention-group';
import { FOLDED_GROUPS, GROUP_LABELS, GROUP_ORDER } from './board.constants';
import type { BoardGroup, BoardLive, BoardRecord } from './board.type';

const groupBoards = (records: readonly BoardRecord[], liveOf?: (record: BoardRecord) => BoardLive | undefined): BoardGroup[] =>
  GROUP_ORDER.map((id) => ({
    id,
    label: GROUP_LABELS[id],
    folded: FOLDED_GROUPS.includes(id),
    boards: records.filter((record) => attentionGroup(record, liveOf?.(record)) === id),
  })).filter((group) => group.boards.length > 0);

export { groupBoards };
