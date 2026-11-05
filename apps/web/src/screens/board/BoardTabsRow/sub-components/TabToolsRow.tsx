/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { OverflowRowItem } from '@drizztdourden08/tessera';
import { toolsRegistry } from '../behavior/tools-registry';
import { useFiltersOpen } from '../behavior/useFiltersOpen';
import { useRegistry } from '../behavior/useRegistry';
import type { TabToolsRowProps } from '../BoardTabsRow.type';

const nodeOf = (item: OverflowRowItem) => (typeof item.node === 'function' ? item.node({ tight: false }) : item.node);

const TabToolsRow = ({ tab }: TabToolsRowProps) => {
  const [open] = useFiltersOpen(tab);
  const items = useRegistry(toolsRegistry).filter((entry) => entry.value.tab === tab).flatMap((entry) => entry.value.items);
  if (!open || items.length === 0) return null;
  return (
    <Flex gap="sm" align="center" wrap role="group" aria-label="View">
      {items.map((item) => <Flex key={item.id} inline align="center">{nodeOf(item)}</Flex>)}
    </Flex>
  );
};

export { TabToolsRow };
