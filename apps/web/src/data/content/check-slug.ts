/* @layer renderer-app @kind logic */
import type { ItemField } from '../registry';
import { SLUG_PATTERN } from './content.constants';
import type { Verdict } from './content.type';
import { reject } from './reject';
import { within } from './within';

const checkSlug = (field: string, spec: ItemField, raw: string): Verdict => {
  const text = raw.trim();
  const tooLong = within(field, text, spec.max);
  if (tooLong) return tooLong;
  if (!SLUG_PATTERN.test(text)) return reject('bad-slug', `${field} is lower case words joined by hyphens, like release-2.`);
  return { ok: true, value: text };
};

export { checkSlug };
