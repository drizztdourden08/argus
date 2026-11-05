/* @layer renderer-app @kind logic */
import type { OverflowRowItem } from '@drizztdourden08/tessera';
import { ANY_TAB, FILTERS_ITEM_ID, TABS_ITEM_ID } from '../BoardTabsRow.constants';
import type { BoardTabsRowProps } from '../BoardTabsRow.type';
import { BoardTabs } from '../sub-components/BoardTabs';
import { FiltersToggle } from '../sub-components/FiltersToggle';
import { filterRegistry } from './filter-registry';
import { toolsRegistry } from './tools-registry';
import { useFiltersOpen } from './useFiltersOpen';
import { useRegistry } from './useRegistry';

const useTabRowItems = (props: BoardTabsRowProps): OverflowRowItem[] => {
  const { activeTab } = props;
  const tools = useRegistry(toolsRegistry);
  const filters = useRegistry(filterRegistry).find((entry) => entry.value.tab === activeTab);
  const [open, setOpen] = useFiltersOpen(activeTab);
  const own = tools.filter((entry) => entry.value.tab === activeTab || entry.value.tab === ANY_TAB).flatMap((entry) => entry.value.items);
  const tabs: OverflowRowItem = { id: TABS_ITEM_ID, pinned: true, node: ({ tight }) => <BoardTabs {...props} tight={tight} /> };
  if (!filters) return [tabs, ...own];
  const toggle = <FiltersToggle open={open} count={filters.value.count} onToggle={() => setOpen(!open)} />;
  return [tabs, ...own, { id: FILTERS_ITEM_ID, pinned: true, node: toggle }];
};

export { useTabRowItems };
