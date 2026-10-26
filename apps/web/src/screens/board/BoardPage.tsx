/* @layer renderer-app @kind component */
import { ContentHeader, EmptyState, Tabs } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { useBoard } from '../../data/react';
import { resolvePalette, useDocumentPalette } from '../../palette';
import { PageFrame, StatusNotice, useHashTab } from '../../shell';
import { BOARD_TAB_IDS, BOARD_TABS } from './board.constants';
import { BoardBody } from './BoardBody';
import type { BoardPageProps } from './board.type';
import { AgentsTab } from './tabs/agents';
import { BuildsTab } from './tabs/builds';
import { TestsTab } from './tabs/tests';
import { WorktreesTab } from './tabs/worktrees';

const BoardPage = (props: BoardPageProps) => {
  const { boardKey } = props;
  const board = useBoard(boardKey);
  const navigate = useNavigate();
  const { tab, select } = useHashTab(BOARD_TAB_IDS, 'agents');
  useDocumentPalette(resolvePalette(boardKey, board.meta?.palette));
  return (
    <PageFrame>
      <ContentHeader title={board.name} back={{ label: 'Argus', onSelect: () => void navigate('/') }} />
      <StatusNotice status={board.status} />
      <BoardBody boardKey={boardKey} />
      <Tabs tabs={BOARD_TABS} activeTab={tab} onTabChange={select} />
      {tab === 'agents' && <AgentsTab boardKey={boardKey} />}
      {tab === 'tests' && <TestsTab boardKey={boardKey} />}
      {tab === 'builds' && <BuildsTab boardKey={boardKey} />}
      {tab === 'worktrees' && <WorktreesTab boardKey={boardKey} />}
      {!['agents', 'tests', 'builds', 'worktrees'].includes(tab) && <EmptyState message="This tab is not built yet." />}
    </PageFrame>
  );
};

export { BoardPage };
