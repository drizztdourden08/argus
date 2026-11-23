/* @layer renderer-app @kind logic */
import { useTabView } from '../../../BoardTabsRow';
import { useStoredFlag, useTabLayout } from '../../../shared';
import { LAYOUT_IDS, WORKTREE_VIEW, WORKTREES_TAB } from '../worktrees.constants';
import type { TabPrefs } from '../worktrees.type';

const useTabPrefs = (boardKey: string): TabPrefs => {
  const [needOnly, setNeedOnly] = useStoredFlag(`argus.tab.${WORKTREES_TAB}.need.${boardKey}`);
  const [compact, setCompact] = useStoredFlag(`argus.tab.${WORKTREES_TAB}.compact.${boardKey}`);
  const [layout, setLayout] = useTabLayout(WORKTREES_TAB, boardKey, LAYOUT_IDS, 'list');
  const view = useTabView(WORKTREES_TAB, WORKTREE_VIEW);
  return { needOnly, setNeedOnly, compact, setCompact, layout, setLayout, view };
};

export { useTabPrefs };
