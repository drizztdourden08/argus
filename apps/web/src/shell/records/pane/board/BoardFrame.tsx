/* @layer renderer-app @kind component */
import { Flex, Icon, Tabs, Text } from '@drizztdourden08/tessera';
import type { TabItem } from '@drizztdourden08/tessera';
import { RecordDrawer, WIDE_KEY, useStoredFlag } from '../../../../screens/board/shared';
import { useNow } from '../../../../screens/board/tabs/kanban/behavior/useNow';
import { boardRecord } from '../../../../screens/overview/boards/model';
import { useRecordPane } from '../../behavior/useRecordPane';
import { usePaneTab } from '../behavior/usePaneTab';
import { RecordPaneTools } from '../RecordPaneTools';
import { useBoardLive } from './behavior/useBoardLive';
import { BoardArchiveTool } from './BoardArchiveTool';
import { BOARD_KIND, BOARD_LABEL } from './board.constants';
import { BOARD_TAB_IDS, BOARD_TABS } from './board-tabs.constants';
import type { BoardFrameProps } from './board.type';

const TAB_ITEMS: TabItem[] = BOARD_TAB_IDS.map((id) => ({ id, label: BOARD_TABS[id]?.label ?? id }));

const BoardFrame = (props: BoardFrameProps) => {
  const { item } = props;
  const pane = useRecordPane();
  const now = useNow();
  const live = useBoardLive(item.doc.key);
  const [wide, setWide] = useStoredFlag(WIDE_KEY);
  const [active, select] = usePaneTab(BOARD_KIND, BOARD_TAB_IDS);
  const record = boardRecord(item, now);
  const Body = BOARD_TABS[active]?.Body;
  const data = { ...live, record, card: item.card, code: item.doc.id, now };
  return (
    <RecordDrawer
      open
      label={BOARD_LABEL}
      wide={wide}
      eyebrow={
        <Flex gap="sm" align="center">
          <Icon name="layout-dashboard" />
          <Text as="span" variant="caption" tone="muted" mono>{data.code}</Text>
        </Flex>
      }
      title={<Text weight={600}>{record.name}</Text>}
      tools={
        <RecordPaneTools code={data.code} pane={pane} previous={null} next={null} wide={wide} onWide={() => setWide(!wide)}>
          <BoardArchiveTool record={record} />
        </RecordPaneTools>
      }
      onClose={pane.close}
    >
      <Flex direction="column" gap="md">
        <Tabs tabs={TAB_ITEMS} activeTab={active} onTabChange={select} />
        <Flex direction="column" role="tabpanel" aria-label={BOARD_TABS[active]?.label}>
          {Body && <Body data={data} />}
        </Flex>
      </Flex>
    </RecordDrawer>
  );
};

export { BoardFrame };
