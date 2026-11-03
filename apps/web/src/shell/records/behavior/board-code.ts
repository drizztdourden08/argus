/* @layer renderer-app @kind logic */
import type { RecordRef } from '../records.type';
import { parseRecord } from './parse-record';

const boardCode = (ref: RecordRef): string => (ref.board ? `${ref.board}/${ref.code}` : ref.code);

const stickyCode = (next: string, here: string | null, current: RecordRef | null): string => {
  const board = current?.board ?? null;
  if (here !== null || board === null) return next;
  return parseRecord(next)?.board === null ? `${board}/${next}` : next;
};

export { boardCode, stickyCode };
