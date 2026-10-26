/* @layer renderer-app @kind component */
import { ContentHeader, EmptyState, Tabs } from '@drizztdourden08/tessera';
import { useLocation, useNavigate } from 'react-router';
import { useBoard } from '../../data/react';
import { resolvePalette, useDocumentPalette } from '../../palette';
import { PageFrame, StatusNotice, useHashTab } from '../../shell';
import { BOARD_TAB_IDS, BOARD_TABS } from './board.constants';
import { BoardBody } from './BoardBody';
import type { BoardPageProps } from './board.type';
import { AgentsTab } from './tabs/agents';
import { BuildsTab } from './tabs/builds';
import { KanbanTab } from './tabs/kanban';
import { ANCHOR_PATTERN, TestsTab, useTestsBadge } from './tabs/tests';
import { WorktreesTab } from './tabs/worktrees';

const BoardPage = (props: BoardPageProps) => {
  const { boardKey } = props;
  const board = useBoard(boardKey);
  const navigate = useNavigate();
  const { hash } = useLocation();
  const picked = useHashTab(BOARD_TAB_IDS, 'agents');
  const tab = ANCHOR_PATTERN.test(hash.slice(1)) ? 'tests' : picked.tab;
  const toRun = useTestsBadge(boardKey);
  const tabs = BOARD_TABS.map((entry) => (entry.id === 'tests' ? { ...entry, badge: toRun } : entry));
  useDocumentPalette(resolvePalette(boardKey, board.meta?.palette));
  return (
    <PageFrame>
      <ContentHeader title={board.name} back={{ label: 'Argus', onSelect: () => void navigate('/') }} />
      <StatusNotice status={board.status} />
      <BoardBody boardKey={boardKey} />
      <Tabs tabs={tabs} activeTab={tab} hideZero onTabChange={picked.select} />
      {tab === 'agents' && <AgentsTab boardKey={boardKey} />}
      {tab === 'kanban' && <KanbanTab boardKey={boardKey} />}
      {tab === 'tests' && <TestsTab boardKey={boardKey} />}
      {tab === 'builds' && <BuildsTab boardKey={boardKey} />}
      {tab === 'worktrees' && <WorktreesTab boardKey={boardKey} />}
      {!['agents', 'kanban', 'tests', 'builds', 'worktrees'].includes(tab) && <EmptyState message="This tab is not built yet." />}
    </PageFrame>
  );
};

export { BoardPage };
