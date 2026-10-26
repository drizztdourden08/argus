/* @layer renderer-app @kind constants */
const BOARD_TABS = [
  { id: 'agents', label: 'Agents' },
  { id: 'activity', label: 'Activity' },
  { id: 'kanban', label: 'Kanban' },
  { id: 'tests', label: 'Tests' },
  { id: 'builds', label: 'Builds' },
  { id: 'worktrees', label: 'Worktrees' },
];

const BOARD_TAB_IDS = BOARD_TABS.map((tab) => tab.id);

const BOARD_KEY_PATTERN = /^[a-z0-9][a-z0-9-]*$/;

export { BOARD_KEY_PATTERN, BOARD_TAB_IDS, BOARD_TABS };
