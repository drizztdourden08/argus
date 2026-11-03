/* @layer renderer-app @kind logic */
import type { ItemField } from '../../registry';
import type { Rejected, RichLine } from '../content.type';
import { reject } from '../reject';

const textLength = (line: RichLine): number => (line.kind === 'image' ? 0 : line.text.length);

const refuseLong = (field: string, spec: ItemField, lines: readonly RichLine[]): Rejected | null => {
  const total = lines.reduce((sum, line) => sum + textLength(line), 0);
  if (spec.max === undefined || total <= spec.max) return null;
  return reject('rich-text-long', `${field} has ${total} chars of text, the limit is ${spec.max}. Shorten it; images do not count.`);
};

export { refuseLong };
