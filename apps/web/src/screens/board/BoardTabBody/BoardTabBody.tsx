/* @layer renderer-app @kind component */
import { Suspense } from 'react';
import { LoadingFallback } from '../../../shell';
import type { BoardTabBodyProps } from './BoardTabBody.type';
import { LazyActivityTab, LazyAgentsTab, LazyBuildsTab, LazyChangesTab, LazyKanbanTab, LazyReleasesTab, LazyTestsTab, LazyWorktreesTab } from './behavior';

const BoardTabBody = (props: BoardTabBodyProps) => {
  const { tab, boardKey, work, handed, more } = props;
  return (
    <Suspense fallback={<LoadingFallback />}>
      {tab === 'agents' && <LazyAgentsTab boardKey={boardKey} onPurge={(thread) => more.openPurge({ scope: 'thread', thread })} />}
      {tab === 'activity' && <LazyActivityTab boardKey={boardKey} work={work} handed={handed} />}
      {tab === 'kanban' && <LazyKanbanTab boardKey={boardKey} trash={more.trashOpen} onTrashBack={more.closeTrash} onPurge={(scope) => more.openPurge({ scope })} />}
      {tab === 'tests' && <LazyTestsTab boardKey={boardKey} />}
      {tab === 'builds' && <LazyBuildsTab boardKey={boardKey} />}
      {tab === 'worktrees' && <LazyWorktreesTab boardKey={boardKey} />}
      {tab === 'changes' && <LazyChangesTab boardKey={boardKey} />}
      {tab === 'releases' && <LazyReleasesTab boardKey={boardKey} />}
    </Suspense>
  );
};

export { BoardTabBody };
