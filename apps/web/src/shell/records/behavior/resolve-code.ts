/* @layer renderer-app @kind logic */
import { ITEM_STORED, kindByCode, kindOf } from '../../../data/registry';
import type { ItemLookup } from '../records.type';
import { ITEM_FALLBACK_KIND } from './record-grammar.constants';

const WRITTEN = /^([A-Z]{1,2})-(\d+)$/;

const resolveCode = (code: string, lookup: ItemLookup): string | null => {
  const found = WRITTEN.exec(code);
  const kind = found ? kindByCode(found[1] ?? '') : undefined;
  if (!found || !kind || kind.family !== 'type' || kind.collection !== 'items') return code;
  const stored = `${ITEM_STORED}-${found[2]}`;
  const item = lookup(stored);
  if (!item) return stored;
  return kindOf(item.type ?? ITEM_FALLBACK_KIND)?.code === kind.code ? stored : null;
};

export { resolveCode };
