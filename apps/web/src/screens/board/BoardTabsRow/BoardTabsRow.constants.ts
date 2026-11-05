/* @layer renderer-app @kind constants */
import type { IconName } from '@drizztdourden08/tessera';

const FILTERS_OPEN_KEY = 'argus.filters.open';

const FILTERS_LABEL = 'Filters';

const LIST_OPERATOR = 'anyOf';

const FLAG_OPERATOR = 'isTrue';

const DEFAULT_TAB_ICON: IconName = 'folder';

const TAB_ICONS: Readonly<Record<string, IconName>> = {
  agents: 'robot',
  activity: 'scroll-text',
  kanban: 'columns-3',
  tests: 'flask',
  builds: 'package',
  worktrees: 'git-branch',
  changes: 'git-fork',
  releases: 'tag',
};

export { DEFAULT_TAB_ICON, FILTERS_LABEL, FILTERS_OPEN_KEY, FLAG_OPERATOR, LIST_OPERATOR, TAB_ICONS };
