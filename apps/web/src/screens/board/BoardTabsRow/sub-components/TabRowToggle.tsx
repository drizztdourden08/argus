/* @layer renderer-app @kind component */
import { filterRegistry } from '../behavior/filter-registry';
import { toolsRegistry } from '../behavior/tools-registry';
import { useFiltersOpen } from '../behavior/useFiltersOpen';
import { useRegistry } from '../behavior/useRegistry';
import type { TabRowToggleProps } from '../BoardTabsRow.type';
import { FiltersToggle } from './FiltersToggle';

const TabRowToggle = ({ tab }: TabRowToggleProps) => {
  const filters = useRegistry(filterRegistry).find((entry) => entry.value.tab === tab);
  const hasTools = useRegistry(toolsRegistry).some((entry) => entry.value.tab === tab && entry.value.items.length > 0);
  const [open, setOpen] = useFiltersOpen(tab);
  if (!filters && !hasTools) return null;
  return <FiltersToggle open={open} count={filters?.value.count ?? 0} onToggle={() => setOpen(!open)} />;
};

export { TabRowToggle };
