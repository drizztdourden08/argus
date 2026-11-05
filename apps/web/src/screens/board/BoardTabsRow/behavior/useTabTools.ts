/* @layer renderer-app @kind logic */
import type { OverflowRowItem } from '@drizztdourden08/tessera';
import { toolsRegistry } from './tools-registry';
import { useRegistered } from './useRegistered';

const useTabTools = (tab: string, items: readonly OverflowRowItem[]): void => useRegistered(toolsRegistry, { tab, items });

export { useTabTools };
