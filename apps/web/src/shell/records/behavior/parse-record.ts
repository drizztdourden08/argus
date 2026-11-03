/* @layer renderer-app @kind logic */
import { CODE_KIND, ITEM_CODE, RECORD_PARTS } from '../records.constants';
import type { RecordRef } from '../records.type';

const parseRecord = (token: string): RecordRef | null => {
  const found = RECORD_PARTS.exec(token);
  if (!found) return null;
  const [, board = null, code = '', digits = ''] = found;
  const kind = CODE_KIND[code];
  if (!kind) return null;
  return { kind, board, id: `${kind === 'item' ? ITEM_CODE : code}-${digits}` };
};

export { parseRecord };
