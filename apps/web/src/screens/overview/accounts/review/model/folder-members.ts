/* @layer renderer-app @kind logic */
import type { ReviewRow } from '../../../../../data/types';

const folderMembers = (rows: readonly ReviewRow[], account: string): Map<string, string[]> => {
  const members = new Map<string, string[]>();
  let current: string[] | null = null;
  for (const row of rows) {
    if (row.kind === 'folder') {
      current = [];
      members.set(row.id, current);
    } else if (current && row.cells[account]?.present) {
      current.push(row.id);
    }
  }
  return members;
};

export { folderMembers };
