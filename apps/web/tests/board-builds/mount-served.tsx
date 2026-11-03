/* @layer renderer-app @kind test */
import { ServedTab } from '../../src/shell/records/pane/tabs/ServedTab';
import { stubFetch } from '../board-tests/mount-tab';
import { itemRef } from '../board-kanban/kanban-docs';
import { paneData } from '../board-kanban/pane-data';
import { SCOPE, mountBoard } from './mount-board';

interface Board {
  launchers?: object[];
  launches?: object[];
  worktrees?: object[];
}

const mountServed = async (itemId: string, board: Board = {}) => {
  stubFetch(undefined);
  const data = paneData({ boardKey: SCOPE, item: itemRef(itemId) });
  const { view } = await mountBoard(<ServedTab data={data} />, [
    { coll: 'launchers', docs: board.launchers ?? [], key: 'name' },
    { coll: 'launches', docs: board.launches ?? [], key: 'name' },
    { coll: 'runs', docs: [], key: 'id' },
    { coll: 'builds', docs: [], key: 'tag' },
    { coll: 'worktrees', docs: board.worktrees ?? [], key: 'id' },
  ]);
  return view;
};

export { mountServed };
