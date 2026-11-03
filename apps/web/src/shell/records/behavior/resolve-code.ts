/* @layer renderer-app @kind logic */
import { ITEM_STORED, kindOf } from '../../../data/registry';
import type { ItemLookup } from '../records.type';
import { ITEM_FALLBACK_KIND } from './record-grammar.constants';
import { typedItemCode } from './typed-item-code';

const resolveCode = (code: string, lookup: ItemLookup): string | null => {
  const typed = typedItemCode(code);
  if (!typed) return code;
  const stored = `${ITEM_STORED}-${typed.number}`;
  const item = lookup(stored);
  if (!item) return stored;
  return kindOf(item.type ?? ITEM_FALLBACK_KIND)?.code === typed.kind.code ? stored : null;
};

export { resolveCode };
