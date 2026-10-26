/* @layer renderer-app @kind constants */
const OVERVIEW_TABS = [
  { id: 'dashboards', label: 'Dashboards' },
  { id: 'processes', label: 'Processes', badgeTitle: 'Processes that need a look' },
  { id: 'activity', label: 'Activity', badgeTitle: 'Warnings and errors since this tab was last open' },
  { id: 'notifications', label: 'Notifications', badgeTitle: 'Notifications since this tab was last open' },
];

const OVERVIEW_TAB_IDS = OVERVIEW_TABS.map((tab) => tab.id);

const CHIP_ALL = '*';

const CHIP_LOOK = { variant: 'normal', color: 'neutral' } as const;

const DOWN_SENTENCE = 'Dashboard unreachable, retrying.';

const ACTIVITY_SEEN_KEY = 'argus.activity.seen';
const NOTIFICATIONS_SEEN_KEY = 'argus.notifications.seen';

export { ACTIVITY_SEEN_KEY, CHIP_ALL, CHIP_LOOK, DOWN_SENTENCE, NOTIFICATIONS_SEEN_KEY, OVERVIEW_TAB_IDS, OVERVIEW_TABS };
