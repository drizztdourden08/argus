/* @layer renderer-app @kind logic */
import type { TabFilterInfo } from '../BoardTabsRow.type';
import { createRegistry } from './create-registry';

const filterRegistry = createRegistry<TabFilterInfo>();

export { filterRegistry };
