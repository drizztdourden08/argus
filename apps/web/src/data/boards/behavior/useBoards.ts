/* @layer renderer-app @kind logic */
import { useEffect, useSyncExternalStore } from 'react';
import { boardsStore } from '../boards-store';
import type { BoardInfo } from '../boards.type';

const useBoards = (): BoardInfo[] => {
  useEffect(() => boardsStore.ensure(), []);
  return useSyncExternalStore(boardsStore.subscribe, boardsStore.get);
};

export { useBoards };
