/* @layer renderer-app @kind component */
import { Flex, OverflowRow, Text } from '@drizztdourden08/tessera';
import type { OverflowRowItem } from '@drizztdourden08/tessera';
import type { ProjectThread } from '../../../../data/agents';
import { useCollection, useDoc } from '../../../../data/react';
import { useRemoteSyncs } from '../../../../data/remote';
import type { BoardMeta } from '../../../../data/types';
import type { WorktreesMeta } from '../../../../data/worktrees';
import { BoardIcon } from '../../../../shell/BoardIcon';
import { BoardSettingsButton } from '../../BoardSettings';
import { RegisteredStrip } from '../../RegisteredStrip';
import { findLead } from '../../tabs/agents/model';
import type { BoardHeaderProps } from '../BoardHeader.type';
import { BoardAlerts } from './BoardAlerts';
import { BoardRunning } from './BoardRunning';
import { BoardStamp } from './BoardStamp';
import { BoardSync } from './BoardSync';
import { BoardWorktrees } from './BoardWorktrees';
import { LeadStatus } from './LeadStatus';

const BoardHeaderRow = (props: BoardHeaderProps) => {
  const { boardKey, name, status, updatedAt, tools } = props;
  const threads = useCollection<ProjectThread>(boardKey, 'threads');
  const { doc } = useDoc<BoardMeta>(boardKey, 'meta', 'board');
  const counts = useDoc<WorktreesMeta>(boardKey, 'meta', 'worktrees').doc;
  const syncs = useRemoteSyncs();
  const listed = counts?.listed;
  const runningAgents = counts?.runningAgents;
  const items: OverflowRowItem[] = [
    {
      id: 'board',
      pinned: true,
      node: (
        <Flex align="center" gap="sm">
          <BoardIcon size="md" boardKey={boardKey} icon={doc?.icon} avatar={doc?.avatar} />
          <Text as="h1" className="board-header__title">{name}</Text>
        </Flex>
      ),
    },
    { id: 'lead', priority: 4, node: <LeadStatus lead={findLead(threads.docs)} /> },
    { id: 'updated', priority: 2, node: <BoardStamp status={status} updatedAt={updatedAt} /> },
    ...(typeof listed === 'number' && typeof runningAgents === 'number' ? [
      { id: 'worktrees', priority: 2, node: <BoardWorktrees listed={listed} runningAgents={runningAgents} /> },
      { id: 'running', priority: 1, node: <BoardRunning runningAgents={runningAgents} /> },
    ] : []),
    ...(syncs ? [{ id: 'sync', priority: 1, node: <BoardSync boardKey={boardKey} /> }] : []),
    { id: 'registered', priority: 3, node: <RegisteredStrip boardKey={boardKey} /> },
    { id: 'alerts', priority: 5, node: <BoardAlerts boardKey={boardKey} status={status} /> },
    { id: 'settings', pinned: true, node: <BoardSettingsButton boardKey={boardKey} /> },
    { id: 'more', pinned: true, node: tools },
  ];
  return <OverflowRow items={items} />;
};

export { BoardHeaderRow };
