/* @layer renderer-app @kind logic */
import { useBoardItems } from '../../../../../data/boards';
import type { BoardLookup } from '../board.type';
import type { RecordRef } from '../../../records.type';

const useBoardRecordData = (target: RecordRef): BoardLookup => {
  const { ready, items } = useBoardItems();
  const item = items.find((candidate) => candidate.doc.id === target.id) ?? null;
  return { item, ready };
};

export { useBoardRecordData };
