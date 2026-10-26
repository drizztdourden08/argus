/* @layer renderer-app @kind logic */
import type { BackupInfo, RemoveResult } from '../../../../../data/worktrees';
import type { Notice, WorktreeRow } from './model.type';
import { nameOf } from './name-of';

const branchText = (result: RemoveResult): string => {
  const branch = result.branch;
  if (!branch) return '';
  if (result.deletedBranch) return `, branch ${branch} deleted${result.branchWhy ? ` (${result.branchWhy})` : ''}`;
  return `, branch ${branch} kept${result.branchNote ? ` (${result.branchNote})` : ''}`;
};

const backupPath = (backup: BackupInfo): string | null => backup.bundle ?? backup.patch ?? backup.dir ?? null;

const removalNotice = (row: WorktreeRow, result: RemoveResult): Notice => {
  const backup = result.backup ?? null;
  const text = `${result.prunedOnly ? 'Pruned' : 'Removed'} ${nameOf(row)}${branchText(result)}.`;
  return {
    text: backup ? `${text} Backup:` : text,
    backup: backup ? backupPath(backup) : null,
    restore: backup?.restore?.[0] ?? null,
  };
};

export { removalNotice };
