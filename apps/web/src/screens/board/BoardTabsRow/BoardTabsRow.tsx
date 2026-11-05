/* @layer renderer-app @kind component */
import { Box, OverflowRow } from '@drizztdourden08/tessera';
import { filtersSlot } from './behavior/filters-slot';
import { useTabRowItems } from './behavior/useTabRowItems';
import type { BoardTabsRowProps } from './BoardTabsRow.type';
import './BoardTabsRow.css';

const BoardTabsRow = (props: BoardTabsRowProps) => {
  const items = useTabRowItems(props);
  return (
    <Box surface="sunken" className="tab-well">
      <OverflowRow className="tab-well__row" items={items} />
      <Box ref={filtersSlot.set} className="tab-well__filters" />
    </Box>
  );
};

export { BoardTabsRow };
