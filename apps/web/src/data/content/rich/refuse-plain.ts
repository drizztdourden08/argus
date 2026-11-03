/* @layer renderer-app @kind logic */
import { RICH_RULE } from '../content.constants';
import type { Rejected, RichLine } from '../content.type';
import { reject } from '../reject';

const LEAD_LATE =
  ': only the first line may be plain text; write the rest as bullets (- point), nested at most two levels, or as images ![caption](img:n).';

const refusePlain = (field: string, lines: readonly RichLine[]): Rejected | null => {
  for (const [index, line] of lines.entries()) {
    if (line.kind !== 'plain') continue;
    if (index > 0) return reject('rich-lead-late', `${field}${LEAD_LATE}`);
    if (line.text.length > RICH_RULE.leadMax) {
      return reject('rich-lead-long', `${field}: the lead line is ${line.text.length} chars, the limit is ${RICH_RULE.leadMax}. Shorten it or make it a bullet.`);
    }
  }
  return null;
};

export { refusePlain };
