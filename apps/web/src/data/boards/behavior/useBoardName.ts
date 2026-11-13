/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { findBoard } from '../find-board';
import { useBoards } from './useBoards';

const useBoardName = (): ((idOrAlias: string) => string) => {
  const boards = useBoards();
  return useCallback((idOrAlias: string) => findBoard(boards, idOrAlias)?.name ?? idOrAlias, [boards]);
};

export { useBoardName };
