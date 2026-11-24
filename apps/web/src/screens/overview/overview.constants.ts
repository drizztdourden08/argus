/* @layer renderer-app @kind constants */
import type { IconName, SegmentOption, SortState } from '@drizztdourden08/tessera';
import type { BoardsLayout } from './boards/boards-view.type';

const OVERVIEW_TABS = [
  { id: 'boards', label: 'Boards' },
  { id: 'processes', label: 'Processes', badgeTitle: 'Processes that need a look' },
  { id: 'accounts', label: 'Accounts', badgeTitle: 'Accounts at a limit' },
  { id: 'activity', label: 'Activity', badgeTitle: 'Warnings and errors since this tab was last open' },
  { id: 'notifications', label: 'Notifications', badgeTitle: 'Notifications since this tab was last open' },
  { id: 'dependencies', label: 'Dependencies', badgeTitle: 'Blocked requests and requests open for over 3 days' },
  { id: 'cache', label: 'Cache' },
  { id: 'releases', label: 'Releases' },
  { id: 'registry', label: 'Registry' },
];

const OVERVIEW_TAB_IDS = OVERVIEW_TABS.map((tab) => tab.id);

const TAB_ALIASES: Readonly<Record<string, string>> = { dashboards: 'boards' };

const OVERVIEW_HASHES = [...OVERVIEW_TAB_IDS, ...Object.keys(TAB_ALIASES)];

const TAB_ICONS: Readonly<Record<string, IconName>> = {
  boards: 'layout-dashboard',
  processes: 'cpu',
  accounts: 'user',
  activity: 'scroll-text',
  notifications: 'bell',
  dependencies: 'link',
  cache: 'database',
  releases: 'tag',
  registry: 'server',
};

const GROUP_OPTIONS = [
  { id: 'attention', label: 'Attention' },
  { id: 'state', label: 'State' },
];

const SORT_OPTIONS = [
  { id: 'attention', label: 'Attention' },
  { id: 'name', label: 'Name' },
  { id: 'updated', label: 'Updated' },
];

const LAYOUT_OPTIONS: SegmentOption<BoardsLayout>[] = [
  { value: 'cards', label: 'Cards' },
  { value: 'rows', label: 'Rows' },
];

const DEFAULT_SORT: SortState = { key: 'attention', direction: 'asc' };

const CHIP_ALL = '*';

const CHIP_LOOK = { variant: 'normal', color: 'neutral' } as const;

const DOWN_SENTENCE = 'Dashboard unreachable, retrying.';

const ACTIVITY_SEEN_KEY = 'argus.activity.seen';
const NOTIFICATIONS_SEEN_KEY = 'argus.notifications.seen';

export {
  ACTIVITY_SEEN_KEY, CHIP_ALL, CHIP_LOOK, DEFAULT_SORT, DOWN_SENTENCE, GROUP_OPTIONS, LAYOUT_OPTIONS, NOTIFICATIONS_SEEN_KEY, OVERVIEW_HASHES, OVERVIEW_TABS, SORT_OPTIONS, TAB_ALIASES, TAB_ICONS,
};
