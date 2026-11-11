/* @layer renderer-app @kind logic */
import type { CompareRow } from '../../../../data/types';
import type { RowMark, SyncStep } from '../compare.type';

const FOLDER_KINDS = new Set(['create_folder', 'write_storage']);

const rowIdOf = (step: SyncStep, rows: readonly CompareRow[]): string | null => {
  if (step.sessionId) {
    const chat = rows.find((row) => row.id === step.sessionId);
    if (chat) return chat.id;
  }
  if (step.folder && (FOLDER_KINDS.has(step.kind ?? '') || !step.sessionId)) {
    return rows.find((row) => row.kind === 'folder' && row.title === step.folder)?.id ?? null;
  }
  return null;
};

const rowMarks = (rows: readonly CompareRow[], steps: readonly SyncStep[]): Map<string, RowMark> => {
  const marks = new Map<string, RowMark>();
  for (const step of steps) {
    const id = rowIdOf(step, rows);
    if (id && step.status) marks.set(id, step.status);
  }
  return marks;
};

export { rowMarks };
