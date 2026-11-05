/* @layer renderer-app @kind constants */
import type { IconName } from '@drizztdourden08/tessera';

const FILTERS_OPEN_KEY = 'argus.filters.open';

const FILTERS_LABEL = 'Filters';


const ANY_VALUE = 'any';

const ANY_LABEL = 'Any';

const TABS_ITEM_ID = 'tabs';

const FILTERS_ITEM_ID = 'filters';

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

export { ANY_LABEL, ANY_VALUE, DEFAULT_TAB_ICON, FILTERS_ITEM_ID, FILTERS_LABEL, FILTERS_OPEN_KEY, TAB_ICONS, TABS_ITEM_ID };
