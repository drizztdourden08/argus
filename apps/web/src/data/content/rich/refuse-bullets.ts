/* @layer renderer-app @kind logic */
import { RICH_RULE } from '../content.constants';
import type { Rejected, RichLine } from '../content.type';
import { reject } from '../reject';

const refuseBullets = (field: string, lines: readonly RichLine[]): Rejected | null => {
  const bullets = lines.flatMap((line) => (line.kind === 'bullet' ? [line.text] : []));
  if (bullets.length > RICH_RULE.bullets) {
    return reject('rich-bullets-many', `${field} has ${bullets.length} bullets, the limit is ${RICH_RULE.bullets}. Merge some.`);
  }
  const long = bullets.find((text) => text.length > RICH_RULE.bulletMax);
  if (long === undefined) return null;
  return reject('rich-bullet-long', `${field}: a bullet is ${long.length} chars, the limit is ${RICH_RULE.bulletMax}. Shorten it.`);
};

export { refuseBullets };
