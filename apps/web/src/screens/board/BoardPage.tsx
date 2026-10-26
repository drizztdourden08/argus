/* @layer renderer-app @kind component */
import { ContentHeader, EmptyState, Tabs } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { useBoard } from '../../data/react';
import { resolvePalette, useDocumentPalette } from '../../palette';
import { PageFrame, StatusNotice } from '../../shell';
import { useBoardTab } from './behavior/useBoardTab';
import { BOARD_TABS } from './board.constants';
import { BoardBody } from './BoardBody';
import { BoardMore, useBoardMore } from './BoardMore';
import type { BoardPageProps } from './board.type';
import { AgentsTab } from './tabs/agents';
import { BuildsTab } from './tabs/builds';
import { KanbanPurge, KanbanTab, useTrashCount } from './tabs/kanban';
import { TestsTab, useTestsBadge } from './tabs/tests';
import { WorktreesTab } from './tabs/worktrees';

const BoardPage = (props: BoardPageProps) => {
  const { boardKey } = props;
  const board = useBoard(boardKey);
  const navigate = useNavigate();
  const more = useBoardMore();
  const { tab, select } = useBoardTab(more.trashOpen);
  const trashCount = useTrashCount(boardKey);
  const toRun = useTestsBadge(boardKey);
  const tabs = BOARD_TABS.map((entry) => (entry.id === 'tests' ? { ...entry, badge: toRun } : entry));
  useDocumentPalette(resolvePalette(boardKey, board.meta?.palette));
  return (
    <PageFrame>
      <ContentHeader
        title={board.name}
        back={{ label: 'Argus', onSelect: () => void navigate('/') }}
        actions={<BoardMore trashCount={trashCount} onTrash={more.openTrash} onPurge={() => more.openPurge({ scope: 'thread' })} />}
      />
      <StatusNotice status={board.status} />
      <BoardBody boardKey={boardKey} />
      <Tabs tabs={tabs} activeTab={tab} hideZero onTabChange={(next) => {
          more.closeTrash();
          select(next);
        }}
      />
      {tab === 'agents' && <AgentsTab boardKey={boardKey} onPurge={(thread) => more.openPurge({ scope: 'thread', thread })} />}
      {tab === 'kanban' && <KanbanTab boardKey={boardKey} trash={more.trashOpen} onTrashBack={more.closeTrash} onPurge={(scope) => more.openPurge({ scope })} />}
      {tab === 'tests' && <TestsTab boardKey={boardKey} />}
      {tab === 'builds' && <BuildsTab boardKey={boardKey} />}
      {tab === 'worktrees' && <WorktreesTab boardKey={boardKey} />}
      {!['agents', 'kanban', 'tests', 'builds', 'worktrees'].includes(tab) && <EmptyState message="This tab is not built yet." />}
      <KanbanPurge boardKey={boardKey} request={more.purge} onClose={more.closePurge} />
    </PageFrame>
  );
};

export { BoardPage };
