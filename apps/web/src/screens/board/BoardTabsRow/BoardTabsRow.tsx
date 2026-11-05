/* @layer renderer-app @kind component */
import { Box, Flex } from '@drizztdourden08/tessera';
import { filtersSlot } from './behavior/filters-slot';
import { BoardTabs } from './sub-components/BoardTabs';
import { TabRowToggle } from './sub-components/TabRowToggle';
import { TabToolsRow } from './sub-components/TabToolsRow';
import type { BoardTabsRowProps } from './BoardTabsRow.type';
import './BoardTabsRow.css';

const BoardTabsRow = (props: BoardTabsRowProps) => (
  <Box surface="sunken" className="tab-well">
    <Flex gap="sm" align="center" className="tab-well__row">
      <Box className="tab-well__tabs">
        <BoardTabs {...props} />
      </Box>
      <TabRowToggle tab={props.activeTab} />
    </Flex>
    <Box ref={filtersSlot.set} className="tab-well__filters" />
    <TabToolsRow tab={props.activeTab} />
  </Box>
);

export { BoardTabsRow };
