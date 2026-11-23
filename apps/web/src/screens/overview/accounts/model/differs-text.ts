/* @layer renderer-app @kind logic */
import { DIFF_KINDS, DIFF_PHRASES, SAME_AS_CURRENT } from '../compare.constants';
import type { CompareDoc } from '../compare.type';
import { plural } from './plural';

const countOf = (compare: CompareDoc, key: string, status: string, kind: string): number =>
  compare.rows.filter((row) => row.kind === kind && row.cells[key]?.status === status).length;

const phraseOf = (compare: CompareDoc, key: string, status: string, kind: string): string[] => {
  const count = countOf(compare, key, status, kind);
  const nouns = DIFF_KINDS[kind];
  const phrase = DIFF_PHRASES[status];
  return count === 0 || !nouns || !phrase ? [] : [phrase(plural(count, nouns[0], nouns[1]))];
};

const differsText = (compare: CompareDoc | null, key: string): string | null => {
  if (!compare) return null;
  const parts = Object.keys(DIFF_PHRASES).flatMap((status) => Object.keys(DIFF_KINDS).flatMap((kind) => phraseOf(compare, key, status, kind)));
  return parts.length === 0 ? SAME_AS_CURRENT : parts.join(', ');
};

export { differsText };
