/* @layer renderer-app @kind test */
import { act } from '@testing-library/react';
import { BuildsTab } from '../../src/screens/board/tabs/builds';
import { stubFetch } from '../board-tests/mount-tab';
import type { Responder } from '../board-tests/mount-tab';
import { SCOPE, mountBoard } from './mount-board';

interface Board {
  launchers?: object[];
  launches?: object[];
  runs?: object[];
  builds?: object[];
  items?: object[];
}

const mountBuilds = async (board: Board = {}, respond?: Responder) => {
  const calls = stubFetch(respond);
  const { view, live } = await mountBoard(<BuildsTab boardKey={SCOPE} />, [
    { coll: 'launchers', docs: board.launchers ?? [], key: 'name' },
    { coll: 'launches', docs: board.launches ?? [], key: 'name' },
    { coll: 'runs', docs: board.runs ?? [], key: 'id' },
    { coll: 'builds', docs: board.builds ?? [], key: 'tag' },
    { coll: 'items', docs: board.items ?? [], key: 'id' },
  ]);
  const change = (coll: string, id: string, data: object) =>
    act(() => live()?.emit('changes', { scope: SCOPE, seq: 99, changes: [{ coll, id, op: 'set', data }] }));
  const remove = (coll: string, id: string) =>
    act(() => live()?.emit('changes', { scope: SCOPE, seq: 100, changes: [{ coll, id, op: 'delete' }] }));
  return { ...view, calls, change, remove };
};

export { mountBuilds };
