/* @layer renderer-app @kind logic */
import { fieldOf } from '../registry';
import { checkList } from './check-list';
import { checkValue } from './check-value';
import type { ContentContext, Verdict } from './content.type';
import { isBlank } from './is-blank';
import { reject } from './reject';

const checkField = (field: string, value: unknown, cx: ContentContext = {}): Verdict => {
  const spec = fieldOf(field);
  if (!spec) return reject('unknown-field', `${field} is not a field of the item registry.`);
  if (isBlank(value)) return spec.required ? reject('required', `${field} is required.`) : { ok: true, value: null };
  return spec.list !== undefined || spec.content === 'refs' ? checkList(field, spec, value, cx) : checkValue(field, spec, value, cx);
};

export { checkField };
