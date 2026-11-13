/* @layer renderer-app @kind logic */
import { useEffect } from 'react';
import { migrateStoredKeys, useBoards } from '../../../data/boards';
import { BOARD_SCOPED_PREFIXES, LAST_BOARD_KEY } from '../AppShell.constants';

const useMigrateStored = (): void => {
  const boards = useBoards();
  useEffect(() => {
    if (boards.length > 0) migrateStoredKeys(window.localStorage, boards, [LAST_BOARD_KEY], BOARD_SCOPED_PREFIXES);
  }, [boards]);
};

export { useMigrateStored };
