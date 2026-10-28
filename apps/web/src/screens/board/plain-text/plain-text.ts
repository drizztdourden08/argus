/* @layer renderer-app @kind logic */
import { smartTimes } from '../../../format';
import {
  BLOCK_MARKS,
  FENCE_PATTERN,
  INLINE_MARKS,
  MAX_CHARS,
  MAX_LINE_CHARS,
  RULE_PATTERN,
  TABLE_RULE_PATTERN,
} from './plain-text.constants';
import type { PlainWalk } from './plain-text.type';

const apply = (line: string, marks: readonly (readonly [RegExp, string])[]): string =>
  marks.reduce((text, [pattern, replacement]) => text.replace(pattern, replacement), line);

const isSkipped = (line: string): boolean =>
  RULE_PATTERN.test(line) || (TABLE_RULE_PATTERN.test(line) && line.includes('|') && line.includes('-'));

const stripLine = (line: string): string => {
  const block = apply(line, BLOCK_MARKS);
  const cells = block.trim().startsWith('|') ? block.replace(/\|/g, ' ') : block;
  return apply(cells, INLINE_MARKS).trim();
};

const closesFence = (found: string | undefined, open: string): boolean =>
  found !== undefined && found.startsWith(open[0] ?? '') && found.length >= open.length;

const step = (walk: PlainWalk, raw: string): PlainWalk => {
  const line = raw.slice(0, MAX_LINE_CHARS);
  const found = FENCE_PATTERN.exec(line)?.[1];
  if (walk.fence) {
    if (closesFence(found, walk.fence)) return { lines: walk.lines, fence: null };
    return { lines: [...walk.lines, line.trim()], fence: walk.fence };
  }
  if (found) return { lines: walk.lines, fence: found };
  if (isSkipped(line)) return walk;
  return { lines: [...walk.lines, stripLine(line)], fence: null };
};

const plainText = (source: unknown): string => {
  const text = (source === null || source === undefined ? '' : String(source)).slice(0, MAX_CHARS);
  const walked = text.replace(/\r\n?/g, '\n').split('\n').reduce(step, { lines: [], fence: null });
  return smartTimes(walked.lines.join(' ').replace(/\s+/g, ' ').trim());
};

export { plainText };
