/* @layer renderer-app @kind logic */
import { useParams } from 'react-router';
import { useOptionalDoc } from '../../../data/react';
import { ownScope } from './own-scope';
import { parseRecord } from './parse-record';
import { shownItemCode } from './shown-item-code';
import { storedItemId } from './stored-item-id';
import { ITEMS } from './useShownCode.constants';

const useShownCode = (id: string, board?: string): string => {
  const here = useParams().key ?? null;
  const ref = parseRecord(id);
  const stored = storedItemId(ref);
  const item = useOptionalDoc<{ type?: string }>(ownScope(stored, ref, board, here), ITEMS, stored ?? '');
  return shownItemCode(id, ref, stored, item);
};

export { useShownCode };
