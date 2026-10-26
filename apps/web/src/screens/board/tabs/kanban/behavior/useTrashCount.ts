/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { trashRows } from '../trash';
import { useKanbanData } from './useKanbanData';

const useTrashCount = (boardKey: string): number => {
  const data = useKanbanData(boardKey);
  return useMemo(() => trashRows(data).length, [data]);
};

export { useTrashCount };
