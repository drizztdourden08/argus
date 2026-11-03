/* @layer renderer-app @kind logic */
import type { RecordRef } from '../records.type';
import { recordSearch } from './record-search';

const recordHref = (ref: RecordRef, here: string | null): string | null => {
  const board = ref.board ?? here;
  return board ? `/p/${encodeURIComponent(board)}${recordSearch('', ref.code, ref.tab)}` : null;
};

export { recordHref };
