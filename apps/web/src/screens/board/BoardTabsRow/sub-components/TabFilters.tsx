/* @layer renderer-app @kind component */
import { Card } from '@drizztdourden08/tessera';
import { createPortal } from 'react-dom';
import { filtersSlot } from '../behavior/filters-slot';
import { useSlotNode } from '../behavior/useSlotNode';
import { useFiltersOpen } from '../behavior/useFiltersOpen';
import { FILTERS_LABEL } from '../BoardTabsRow.constants';
import type { TabFiltersProps } from '../BoardTabsRow.type';
import { FiltersToggle } from './FiltersToggle';
import { TabTools } from './TabTools';

const TabFilters = (props: TabFiltersProps) => {
  const { tab, count, children } = props;
  const [open, setOpen] = useFiltersOpen(tab);
  const row = useSlotNode(filtersSlot);
  return (
    <>
      <TabTools>
        <FiltersToggle open={open} count={count} onToggle={() => setOpen(!open)} />
      </TabTools>
      {open && row && createPortal(<Card container role="search" aria-label={FILTERS_LABEL}>{children}</Card>, row)}
    </>
  );
};

export { TabFilters };
