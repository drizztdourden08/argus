/* @layer renderer-app @kind logic */
import type { OverflowRowItem } from '@drizztdourden08/tessera';
import { FILTERS_ITEM_ID, TABS_ITEM_ID } from '../BoardTabsRow.constants';
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
  const own = tools.filter((entry) => entry.value.tab === activeTab).flatMap((entry) => entry.value.items);
  const tabs: OverflowRowItem = { id: TABS_ITEM_ID, pinned: true, shrink: true, node: ({ tight }) => <BoardTabs {...props} tight={tight} /> };
  const toggle = filters ? <FiltersToggle open={open} count={filters.value.count} onToggle={() => setOpen(!open)} /> : null;
  const rest = toggle ? [...own, { id: FILTERS_ITEM_ID, pinned: true, node: toggle }] : own;
  return [tabs, ...rest.map((item, index) => (index === 0 ? { ...item, end: true } : item))];
};

export { useTabRowItems };
