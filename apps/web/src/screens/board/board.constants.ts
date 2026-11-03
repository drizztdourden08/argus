/* @layer renderer-app @kind constants */
const BOARD_TABS = [
  { id: 'agents', label: 'Agents' },
  { id: 'activity', label: 'Activity' },
  { id: 'kanban', label: 'Kanban' },
  { id: 'tests', label: 'Tests' },
  { id: 'builds', label: 'Builds' },
  { id: 'worktrees', label: 'Worktrees' },
  { id: 'changes', label: 'Changes' },
  { id: 'releases', label: 'Releases' },
];

const BOARD_TAB_IDS = BOARD_TABS.map((tab) => tab.id);

const RETIRED_TABS: ReadonlyMap<string, string> = new Map([['media', 'kanban']]);

const BOARD_KEY_PATTERN = /^[a-z0-9][a-z0-9-]*$/;

export { BOARD_KEY_PATTERN, BOARD_TAB_IDS, BOARD_TABS, RETIRED_TABS };
