/* @layer renderer-app @kind component */
import { Flex, OverflowRow, Tag } from '@drizztdourden08/tessera';
import type { OverflowRowItem } from '@drizztdourden08/tessera';
import { useDoc } from '../../../../data/react';
import type { BoardMeta } from '../../../../data/types';
import { BoardIcon } from '../../../../shell/BoardIcon';
import { BoardSettingsButton } from '../../BoardSettings';
import { useBoardHealth } from '../behavior/useBoardHealth';
import type { BoardHeaderProps } from '../BoardHeader.type';
import { BoardAlerts } from './BoardAlerts';
import { BoardStatus } from './BoardStatus';
import { BoardTitle } from './BoardTitle';

const BoardHeaderRow = (props: BoardHeaderProps) => {
  const { boardKey, name, status, tools } = props;
  const { doc } = useDoc<BoardMeta>(boardKey, 'meta', 'board');
  const health = useBoardHealth(props);
  const faults = health.faults.length;
  const items: OverflowRowItem[] = [
    {
      id: 'board',
      pinned: true,
      node: (
        <Flex align="center" gap="sm">
          <BoardIcon size="md" boardKey={boardKey} icon={doc?.icon} avatar={doc?.avatar} />
          <BoardTitle boardKey={boardKey} name={name} />
        </Flex>
      ),
    },
    { id: 'health', pinned: true, node: <BoardStatus boardKey={boardKey} health={health} /> },
    ...(faults > 0 ? [{ id: 'faults', priority: 1, node: <Tag variant="urgency" color="danger">{`${faults} fault${faults > 1 ? 's' : ''}`}</Tag> }] : []),
    { id: 'alerts', pinned: true, node: <BoardAlerts boardKey={boardKey} status={status} /> },
    { id: 'settings', priority: 2, end: true, node: <BoardSettingsButton boardKey={boardKey} /> },
    { id: 'more', pinned: true, node: tools },
  ];
  return <OverflowRow items={items} />;
};

export { BoardHeaderRow };
