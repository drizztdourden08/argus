/* @layer renderer-app @kind logic */
import type { RecordRef } from '../records.type';
import { recordHash } from './record-hash';

const recordHref = (ref: RecordRef, here: string | null): string | null => {
  const board = ref.board ?? here;
  return board ? `/p/${encodeURIComponent(board)}#${recordHash(ref)}` : null;
};

export { recordHref };
