/* @layer renderer-app @kind logic */
import { ITEM_REGISTRY } from '../registry';
import type { ItemField } from '../registry';
import { LINE_BLOCK } from './content.constants';
import type { Verdict } from './content.type';
import { reject } from './reject';
import { within } from './within';

const NEWLINE = /[\n\r]/;

const checkLine = (field: string, spec: ItemField, raw: string): Verdict => {
  const kind = ITEM_REGISTRY.content.line;
  const text = raw.trim();
  if (!kind.newlines && NEWLINE.test(text)) return reject('newline', `${field} must be one line.`);
  if (!kind.markup && LINE_BLOCK.test(text)) {
    return reject('line-markup', `${field} is plain text: it cannot start with #, |, >, a code fence or a list marker.`);
  }
  return within(field, text, spec.max) ?? { ok: true, value: text };
};

export { checkLine };
