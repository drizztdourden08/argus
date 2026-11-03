/* @layer renderer-app @kind logic */
import type { RecordRef } from '../records.type';
import { itemCode } from './item-code';

const shownItemCode = (id: string, ref: RecordRef | null, stored: string | null, item: { type?: string } | null): string => {
  if (!stored || !item) return id;
  const code = itemCode({ id: stored, type: item.type });
  return ref?.board ? `${ref.board}/${code}` : code;
};

export { shownItemCode };
