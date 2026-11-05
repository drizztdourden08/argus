/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { createPortal } from 'react-dom';
import { filterRegistry } from '../behavior/filter-registry';
import { filtersSlot } from '../behavior/filters-slot';
import { useFiltersOpen } from '../behavior/useFiltersOpen';
import { useRegistered } from '../behavior/useRegistered';
import { useSlotNode } from '../behavior/useSlotNode';
import { FILTERS_LABEL } from '../BoardTabsRow.constants';
import type { TabFilterRowProps } from '../BoardTabsRow.type';

const TabFilterRow = ({ tab, count, children }: TabFilterRowProps) => {
  const [open] = useFiltersOpen(tab);
  const row = useSlotNode(filtersSlot);
  useRegistered(filterRegistry, { tab, count });
  return open && row ? createPortal(<Box role="search" aria-label={FILTERS_LABEL}>{children}</Box>, row) : null;
};

export { TabFilterRow };
