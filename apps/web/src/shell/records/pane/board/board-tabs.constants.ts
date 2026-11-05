/* @layer renderer-app @kind constants */
import type { TabItem } from '@drizztdourden08/tessera';
import { BoardActivityTab } from './BoardActivityTab';
import { BoardBuildsTab } from './BoardBuildsTab';
import { BoardItemsTab } from './BoardItemsTab';
import { BoardLinksTab } from './BoardLinksTab';
import { BoardOverviewTab } from './BoardOverviewTab';
import { BoardTestsTab } from './BoardTestsTab';
import { BoardWorkTab } from './BoardWorkTab';
import { BoardWorktreesTab } from './BoardWorktreesTab';
import type { BoardTab } from './board.type';

const BOARD_TABS: Readonly<Record<string, BoardTab>> = {
  overview: { label: 'Overview', Body: BoardOverviewTab },
  work: { label: 'Work', Body: BoardWorkTab },
  items: { label: 'Items', Body: BoardItemsTab },
  tests: { label: 'Tests', Body: BoardTestsTab },
  worktrees: { label: 'Worktrees', Body: BoardWorktreesTab },
  builds: { label: 'Builds', Body: BoardBuildsTab },
  links: { label: 'Links', Body: BoardLinksTab },
  activity: { label: 'Activity', Body: BoardActivityTab },
};

const BOARD_TAB_IDS: readonly string[] = Object.keys(BOARD_TABS);

const TAB_ITEMS: TabItem[] = BOARD_TAB_IDS.map((id) => ({ id, label: BOARD_TABS[id]?.label ?? id }));

export { BOARD_TAB_IDS, BOARD_TABS, TAB_ITEMS };
