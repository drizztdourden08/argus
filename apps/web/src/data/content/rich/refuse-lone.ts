/* @layer renderer-app @kind logic */
import { LONE_PARAGRAPH, RICH_RULE } from '../content.constants';
import type { Rejected, RichLine } from '../content.type';
import { reject } from '../reject';

const refuseLone = (lines: readonly RichLine[]): Rejected | null => {
  const [first] = lines;
  const marked = lines.some((line) => line.kind !== 'plain');
  const short = lines.length === 1 && first?.kind === 'plain' && first.text.length <= RICH_RULE.shortMax;
  return marked || short ? null : reject('rich-lone-paragraph', LONE_PARAGRAPH);
};

export { refuseLone };
