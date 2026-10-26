/* @layer renderer-app @kind constants */
const OVERVIEW_TABS = [
  { id: 'dashboards', label: 'Dashboards' },
  { id: 'processes', label: 'Processes' },
  { id: 'activity', label: 'Activity' },
  { id: 'notifications', label: 'Notifications' },
];

const OVERVIEW_TAB_IDS = OVERVIEW_TABS.map((tab) => tab.id);

const DOWN_SENTENCE = 'Dashboard unreachable, retrying.';

const ACTIVITY_SEEN_KEY = 'argus.activity.seen';
const NOTIFICATIONS_SEEN_KEY = 'argus.notifications.seen';

export { ACTIVITY_SEEN_KEY, DOWN_SENTENCE, NOTIFICATIONS_SEEN_KEY, OVERVIEW_TAB_IDS, OVERVIEW_TABS };
