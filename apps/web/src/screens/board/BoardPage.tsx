/* @layer renderer-app @kind component */
import { Tabs } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useBoard } from '../../data/react';
import { resolvePalette, useDocumentPalette } from '../../palette';
import { PageFrame, StatusNotice } from '../../shell';
import { useBoardTab } from './behavior/useBoardTab';
import { BOARD_TABS } from './board.constants';
import { BoardFocus } from './BoardFocus';
import { BoardHeader } from './BoardHeader';
import { BoardMore, useBoardMore } from './BoardMore';
import { BoardTabBody } from './BoardTabBody';
import { useBoardWork } from './feed-model';
import { RegisteredStrip } from './RegisteredStrip';
import { OpenThreadContext } from './shared';
import { useTabBadges, withBadges } from './tab-badges';
import { ThreadDrawer } from './ThreadDrawer';
import type { BoardPageProps } from './board.type';
import { KanbanPurge, useTrashCount } from './tabs/kanban';

const BoardPage = (props: BoardPageProps) => {
  const { boardKey } = props;
  const board = useBoard(boardKey);
  const more = useBoardMore();
  const [openThread, setOpenThread] = useState<string | null>(null);
  const { tab, select } = useBoardTab(more.trashOpen);
  const trashCount = useTrashCount(boardKey);
  const work = useBoardWork(boardKey);
  const badges = useTabBadges(boardKey, work);
  useDocumentPalette(resolvePalette(boardKey, board.meta?.palette));
  return (
    <OpenThreadContext.Provider value={setOpenThread}>
      <PageFrame>
        <BoardHeader
          boardKey={boardKey}
          name={board.name}
          status={board.status}
          updatedAt={board.meta?.updatedAt ?? null}
          tools={<BoardMore boardKey={boardKey} trashCount={trashCount} onTrash={more.openTrash} onPurge={() => more.openPurge({ scope: 'thread' })} />}
        />
        <StatusNotice status={board.status} />
        <RegisteredStrip boardKey={boardKey} />
        <Tabs tabs={withBadges(BOARD_TABS, badges)} activeTab={more.trashOpen ? '' : tab} onTabChange={(next) => {
            more.closeTrash();
            select(next);
          }}
        />
        <BoardFocus boardKey={boardKey} />
        <BoardTabBody tab={tab} boardKey={boardKey} work={work} handed={board.meta?.handedBuild ?? null} more={more} />
        <KanbanPurge boardKey={boardKey} request={more.purge} onClose={more.closePurge} />
        <ThreadDrawer boardKey={boardKey} id={openThread} onClose={() => setOpenThread(null)} onPurge={(name) => more.openPurge({ scope: 'thread', thread: name })} />
      </PageFrame>
    </OpenThreadContext.Provider>
  );
};

export { BoardPage };
