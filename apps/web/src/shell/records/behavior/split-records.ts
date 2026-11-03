/* @layer renderer-app @kind logic */
import type { TextPart } from '../records.type';
import { parseRecord } from './parse-record';
import { EVERY_TOKEN } from './split-records.constants';

const splitRecords = (text: string): TextPart[] => {
  const parts: TextPart[] = [];
  let from = 0;
  for (const found of text.matchAll(EVERY_TOKEN)) {
    const ref = parseRecord(found[0]);
    if (ref) {
      if (found.index > from) parts.push({ text: text.slice(from, found.index), ref: null });
      parts.push({ text: found[0], ref });
      from = found.index + found[0].length;
    }
  }
  if (from < text.length) parts.push({ text: text.slice(from), ref: null });
  return parts;
};

export { splitRecords };
