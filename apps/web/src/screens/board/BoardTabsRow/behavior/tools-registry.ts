/* @layer renderer-app @kind logic */
import type { TabToolsEntry } from '../BoardTabsRow.type';
import { createRegistry } from './create-registry';

const toolsRegistry = createRegistry<TabToolsEntry>();

export { toolsRegistry };
