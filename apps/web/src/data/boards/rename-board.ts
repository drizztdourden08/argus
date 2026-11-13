/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { BoardRename } from './boards.type';
import { boardsStore } from './boards-store';

const renameBoard = async (id: string, name: string): Promise<BoardRename> => {
  const renamed = await apiRequest<BoardRename>(`/api/boards/${encodeURIComponent(id)}`, { method: 'PATCH', json: { name } });
  await boardsStore.refresh();
  return renamed;
};

export { renameBoard };
