/* @layer renderer-app @kind constants */
const OVERVIEW_TABS = [
  { id: 'dashboards', label: 'Dashboards' },
  { id: 'processes', label: 'Processes' },
  { id: 'activity', label: 'Activity' },
  { id: 'notifications', label: 'Notifications' },
];

const OVERVIEW_TAB_IDS = OVERVIEW_TABS.map((tab) => tab.id);

const CHIP_ALL = '*';

const CHIP_LOOK = { variant: 'normal', color: 'neutral' } as const;

const DOWN_SENTENCE = 'Dashboard unreachable, retrying.';

const ACTIVITY_SEEN_KEY = 'argus.activity.seen';
const NOTIFICATIONS_SEEN_KEY = 'argus.notifications.seen';

export { ACTIVITY_SEEN_KEY, CHIP_ALL, CHIP_LOOK, DOWN_SENTENCE, NOTIFICATIONS_SEEN_KEY, OVERVIEW_TAB_IDS, OVERVIEW_TABS };
