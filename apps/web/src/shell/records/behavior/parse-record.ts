/* @layer renderer-app @kind logic */
import { PORTFOLIO_SCOPE } from '../../../data/react';
import { kindByCode, kindOf } from '../../../data/registry';
import type { RegistryKind } from '../../../data/registry';
import { RECORD_PARTS, RECORD_TAILS } from '../records.constants';
import type { RecordRef } from '../records.type';
import { ITEM_FALLBACK_KIND, ITEM_LETTER } from './record-grammar.constants';

const kindOfLetters = (letters: string): RegistryKind | undefined => (letters === ITEM_LETTER ? kindOf(ITEM_FALLBACK_KIND) : kindByCode(letters));

const idOf = (kind: RegistryKind, letters: string, rest: string): string => {
  if (kind.idKey !== undefined) return rest;
  if (letters === ITEM_LETTER || kind.collection === 'items') return `${ITEM_LETTER}-${rest}`;
  return `${kind.stored ?? letters}-${rest}`;
};

const scopeOf = (kind: RegistryKind, board: string | null): string | null => (kind.collection === 'boards' ? PORTFOLIO_SCOPE : board);

const parseRecord =(token: string): RecordRef | null => {
  const found = RECORD_PARTS.exec(token);
  if (!found) return null;
  const [, board = null, letters = '', rest = ''] = found;
  const kind = kindOfLetters(letters);
  if (!kind || !RECORD_TAILS[letters]?.test(rest)) return null;
  return { kind: kind.id, collection: kind.collection, code: `${letters}-${rest}`, id: idOf(kind, letters, rest), board: scopeOf(kind, board) };
};

export { parseRecord };
