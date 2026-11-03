/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { useFeedActions } from '../../../OwnerFeed';
import { useItemRecord } from '../behavior/useItemRecord';
import { useRecordActions } from '../behavior/useRecordActions';
import { usePaneTab } from './behavior/usePaneTab';
import { tabsOf } from './model/tabs-of';
import { PaneCrumbs } from './PaneCrumbs';
import { PANE_TABS } from './pane.constants';
import { PaneTabs } from './PaneTabs';
import type { ItemPaneProps, PaneData } from './pane.type';

const ItemPane = (props: ItemPaneProps) => {
  const { boardKey, item, items, tests, threads, actions, waiting, blocking, now, ops, onOpen } = props;
  const record = useItemRecord(boardKey, item.id, actions);
  const acts = useRecordActions(boardKey);
  const feed = useFeedActions(boardKey);
  const tabs = tabsOf(item);
  const [active, select] = usePaneTab(item, tabs);
  const data: PaneData = {
    boardKey, item, items, tests: tests.filter((test) => test.item === item.id), allTests: tests, threads, record, waiting, blocking, now, ops, acts, feed, open: onOpen,
  };
  const Body = PANE_TABS[active]?.Body;
  return (
    <Flex direction="column" gap="md">
      <PaneCrumbs item={item} items={items} onOpen={onOpen} />
      <PaneTabs data={data} tabs={tabs} active={active} onSelect={select} />
      <Flex direction="column" role="tabpanel" aria-label={PANE_TABS[active]?.label}>
        {Body && <Body data={data} />}
      </Flex>
    </Flex>
  );
};

export { ItemPane };
