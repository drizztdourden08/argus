/* @layer renderer-app @kind logic */
import type { ItemField } from '../registry';
import { checkRecords } from './check-records';
import type { ContentContext, Verdict } from './content.type';
import { reject } from './reject';
import { textChecks } from './text-checks';

const checkValue = (field: string, spec: ItemField, value: unknown, cx: ContentContext): Verdict => {
  const kind = spec.content;
  if (kind === 'records') return checkRecords(field, value);
  if (typeof value !== 'string') return reject('not-text', `${field} must be a text.`);
  return textChecks[kind](field, spec, value, cx);
};

export { checkValue };
