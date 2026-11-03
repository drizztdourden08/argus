/* @layer renderer-app @kind logic */
import { useParams } from 'react-router';
import { useOptionalDoc } from '../../../data/react';
import { itemCode } from './item-code';
import { parseRecord } from './parse-record';

const ITEMS = 'items';

const useShownCode = (id: string, board?: string): string => {
  const here = useParams().key ?? null;
  const ref = parseRecord(id);
  const stored = ref && ref.collection === ITEMS && ref.code === ref.id ? ref.id : null;
  const own = stored !== null && (ref?.board ?? board ?? here) === here;
  const item = useOptionalDoc<{ type?: string }>(own ? here : null, ITEMS, stored ?? '');
  if (!stored || !item) return id;
  const code = itemCode({ id: stored, type: item.type });
  return ref?.board ? `${ref.board}/${code}` : code;
};

export { useShownCode };
