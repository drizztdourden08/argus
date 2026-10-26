/* @layer renderer-app @kind constants */
const OVERVIEW_TABS = [
  { id: 'dashboards', label: 'Dashboards' },
  { id: 'processes', label: 'Processes' },
  { id: 'activity', label: 'Activity' },
  { id: 'notifications', label: 'Notifications' },
];

const OVERVIEW_TAB_IDS = OVERVIEW_TABS.map((tab) => tab.id);

export { OVERVIEW_TAB_IDS, OVERVIEW_TABS };
