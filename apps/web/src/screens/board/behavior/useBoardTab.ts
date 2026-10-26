/* @layer renderer-app @kind logic */
import { useLocation } from 'react-router';
import { useHashTab } from '../../../shell';
import type { HashTab } from '../../../shell';
import { BOARD_TAB_IDS } from '../board.constants';
import { itemHash } from '../tabs/kanban';
import { anchorTab } from './anchor-tab';

const useBoardTab = (trashOpen: boolean): HashTab => {
  const { hash } = useLocation();
  const picked = useHashTab(BOARD_TAB_IDS, 'agents');
  const anchored = anchorTab(hash.slice(1)) ?? picked.tab;
  return { tab: trashOpen || itemHash(hash) ? 'kanban' : anchored, select: picked.select };
};

export { useBoardTab };
