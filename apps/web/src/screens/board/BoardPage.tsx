/* @layer renderer-app @kind component */
import { Tabs } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useBoard } from '../../data/react';
import { resolvePalette, useDocumentPalette } from '../../palette';
import { PageFrame, RecordPaneScope, StatusNotice } from '../../shell';
import { RecordPaneHost } from '../../shell/records/pane';
import { useBoardTab } from './behavior/useBoardTab';
import { usePresence } from './behavior/usePresence';
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
import { useTrashCount } from './tabs/kanban/behavior/useTrashCount';
import { KanbanPurge } from './tabs/kanban/sub-components/KanbanPurge';

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
  usePresence(boardKey);
  return (
    <OpenThreadContext.Provider value={setOpenThread}>
      <RecordPaneScope>
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
          <Tabs tabs={withBadges(BOARD_TABS, badges)} activeTab={more.trashOpen ? '' : tab} size="lg" selectedTone="neutral" badgeFill="solid" onTabChange={(next) => {
              more.closeTrash();
              select(next);
            }}
          />
          <BoardFocus boardKey={boardKey} />
          <BoardTabBody tab={tab} boardKey={boardKey} work={work} handed={board.meta?.handedBuild ?? null} more={more} />
          <KanbanPurge boardKey={boardKey} request={more.purge} onClose={more.closePurge} />
          <ThreadDrawer boardKey={boardKey} id={openThread} onClose={() => setOpenThread(null)} onPurge={(name) => more.openPurge({ scope: 'thread', thread: name })} />
          <RecordPaneHost boardKey={boardKey} />
        </PageFrame>
      </RecordPaneScope>
    </OpenThreadContext.Provider>
  );
};

export { BoardPage };
