/* @layer renderer-app @kind logic */
import { kindOf } from '../../../data/registry';
import { ITEM_FALLBACK_KIND } from './record-grammar.constants';
import { shownCode } from './shown-code';

const itemCode = (item: { id: string; type?: string }): string => {
  const kind = kindOf(item.type ?? ITEM_FALLBACK_KIND);
  return kind ? shownCode(kind, item.id) : item.id;
};

export { itemCode };
