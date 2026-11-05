/* @layer renderer-app @kind component */
import { Flex, Tabs } from '@drizztdourden08/tessera';
import type { BoardTabsRowProps } from './BoardTabsRow.type';
import { TabFiltersSlot } from './sub-components/TabFiltersSlot';
import { TabToolsSlot } from './sub-components/TabToolsSlot';

const BoardTabsRow = (props: BoardTabsRowProps) => {
  const { tabs, activeTab, onTabChange } = props;
  return (
    <Flex direction="column" gap="sm">
      <Flex justify="between" align="center" gap="md" wrap>
        <Tabs tabs={tabs} activeTab={activeTab} size="lg" selectedTone="neutral" badgeFill="solid" onTabChange={onTabChange} />
        <TabToolsSlot />
      </Flex>
      <TabFiltersSlot />
    </Flex>
  );
};

export { BoardTabsRow };
