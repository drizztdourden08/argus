/* @layer renderer-app @kind component */
import { Box, Button, Icon, OverflowRow, Text } from '@drizztdourden08/tessera';
import type { OverflowRowItem } from '@drizztdourden08/tessera';
import { useRef } from 'react';
import { SendDownPopover } from '../dashboards/sub-components/SendDownPopover';
import { boardsToolItems } from '../behavior/boards-tool-items';
import type { OverviewHeaderProps } from '../overview.type';
import { NowPopover } from './NowPopover';
import { OverviewAlerts } from './OverviewAlerts';
import { OverviewReadouts } from './OverviewReadouts';
import './OverviewHeader.css';

const OverviewHeader = (props: OverviewHeaderProps) => {
  const { tab, data, view, status, send, wide } = props;
  const anchor = useRef<HTMLElement>(null);
  const onBoards = tab === 'boards';
  const items: OverflowRowItem[] = [
    { id: 'title', pinned: true, node: <Text as="h1" className="overview-header__title">Overview</Text> },
    { id: 'readouts', priority: 6, node: <OverviewReadouts entries={data.entries} status={status} /> },
    { id: 'alerts', priority: 7, node: <OverviewAlerts entries={data.entries} status={status} /> },
    ...(onBoards ? boardsToolItems(view) : []),
    ...(onBoards && !wide ? [{ id: 'now', priority: 5, node: <NowPopover entries={data.entries} /> }] : []),
    {
      id: 'send',
      priority: 1,
      end: !onBoards,
      node: <Button size="sm" icon={<Icon name="send" />} onClick={() => send.open('all')} title="Send a decision or a note to every lead">Send to all</Button>,
    },
  ];
  return (
    <Box ref={anchor}>
      <OverflowRow className="overview-header" items={items} />
      <SendDownPopover send={send} owner="all" anchorRef={anchor} />
    </Box>
  );
};

export { OverviewHeader };
