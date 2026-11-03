/* @layer renderer-app @kind logic */
import { kindOf } from '../../../data/registry';
import type { CodeEntry, RegistryKind } from '../../../data/registry';
import type { RecordGrammar } from '../records.type';
import { BOARD_PREFIX, ITEM_FALLBACK_KIND, ITEM_LETTER, TAIL_NAME, TAIL_NUMBER, TAIL_VERSION } from './record-grammar.constants';

const tailOf = (kind: RegistryKind | undefined): string => {
  if (kind?.idKey === undefined) return TAIL_NUMBER;
  return kind.idKey === 'version' ? TAIL_VERSION : TAIL_NAME;
};

const tailsOf = (table: readonly CodeEntry[]): Map<string, string> => {
  const tails = new Map<string, string>([[ITEM_LETTER, tailOf(kindOf(ITEM_FALLBACK_KIND))]]);
  for (const entry of table) tails.set(entry.letters, tailOf(kindOf(entry.kind)));
  return tails;
};

const buildRecordGrammar = (table: readonly CodeEntry[]): RecordGrammar => {
  const tails = tailsOf(table);
  const longestFirst = [...tails.keys()].sort((a, b) => b.length - a.length || a.localeCompare(b));
  const alternatives = longestFirst.map((letters) => `${letters}-(?:${tails.get(letters)})`);
  const token = new RegExp(`(?<![\\w#/-])${BOARD_PREFIX}(?:${alternatives.join('|')})(?!\\w|-\\w)`);
  const checks: Record<string, RegExp> = {};
  for (const [letters, tail] of tails) checks[letters] = new RegExp(`^(?:${tail})$`);
  return { token, checks };
};

export { buildRecordGrammar };
