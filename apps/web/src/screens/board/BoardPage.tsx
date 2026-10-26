/* @layer renderer-app @kind component */
import { ContentHeader, EmptyState, Tabs } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { useBoard } from '../../data/react';
import { resolvePalette, useDocumentPalette } from '../../palette';
import { PageFrame, StatusNotice, useHashTab } from '../../shell';
import { BOARD_TAB_IDS, BOARD_TABS } from './board.constants';
import type { BoardPageProps } from './board.type';
import { ThreadList } from './sub-components/ThreadList';

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
      <ThreadList threads={board.threads} />
      <Tabs tabs={BOARD_TABS} activeTab={tab} onTabChange={select} />
      <EmptyState message="This tab is not built yet." />
    </PageFrame>
  );
};

export { BoardPage };
