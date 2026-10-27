/* @layer renderer-app @kind component */
import { ActivityTab } from '../tabs/activity';
import { AgentsTab } from '../tabs/agents';
import { BuildsTab } from '../tabs/builds';
import { KanbanTab } from '../tabs/kanban';
import { TestsTab } from '../tabs/tests';
import { WorktreesTab } from '../tabs/worktrees';
import type { BoardTabBodyProps } from './BoardTabBody.type';

const BoardTabBody = (props: BoardTabBodyProps) => {
  const { tab, boardKey, work, handed, more } = props;
  return (
    <>
      {tab === 'agents' && <AgentsTab boardKey={boardKey} onPurge={(thread) => more.openPurge({ scope: 'thread', thread })} />}
      {tab === 'activity' && <ActivityTab boardKey={boardKey} work={work} handed={handed} />}
      {tab === 'kanban' && <KanbanTab boardKey={boardKey} trash={more.trashOpen} onTrashBack={more.closeTrash} onPurge={(scope) => more.openPurge({ scope })} />}
      {tab === 'tests' && <TestsTab boardKey={boardKey} />}
      {tab === 'builds' && <BuildsTab boardKey={boardKey} />}
      {tab === 'worktrees' && <WorktreesTab boardKey={boardKey} />}
    </>
  );
};

export { BoardTabBody };
