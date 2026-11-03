/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { usePaneTab } from './behavior/usePaneTab';
import { kindTabs } from './model/kind-tabs';
import { PaneCrumbs } from './PaneCrumbs';
import { PANE_TABS } from './pane.constants';
import { PaneTabs } from './PaneTabs';
import type { RecordBodyProps } from './pane.type';

const RecordBody = (props: RecordBodyProps) => {
  const { data } = props;
  const tabs = kindTabs(data.kind);
  const [active, select] = usePaneTab(data.kind.id, tabs);
  const Body = PANE_TABS[active]?.Body;
  return (
    <Flex direction="column" gap="md">
      {data.item && <PaneCrumbs item={data.item.item} items={data.item.items} onOpen={(id) => data.open(id)} />}
      <PaneTabs data={data} tabs={tabs} active={active} onSelect={select} />
      <Flex direction="column" role="tabpanel" aria-label={PANE_TABS[active]?.label}>
        {Body && <Body data={data} />}
      </Flex>
    </Flex>
  );
};

export { RecordBody };
