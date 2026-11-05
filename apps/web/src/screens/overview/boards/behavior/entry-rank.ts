/* @layer renderer-app @kind logic */
import type { BoardEntry } from '../boards-view.type';
import { attentionGroup } from '../model';
import { GROUP_ORDER } from '../model/board.constants';

const entryRank = (entry: BoardEntry): number => GROUP_ORDER.indexOf(attentionGroup(entry.record, entry.live));

export { entryRank };
