/* @layer renderer-app @kind logic */
import type { ItemField } from '../registry';
import { checkValue } from './check-value';
import type { ContentContext, Verdict } from './content.type';
import { isBlank } from './is-blank';
import { reject } from './reject';

const checkList = (field: string, spec: ItemField, value: unknown, cx: ContentContext): Verdict => {
  if (!Array.isArray(value)) return reject('not-a-list', `${field} must be a list.`);
  if (spec.list !== undefined && value.length > spec.list) return reject('list-too-long', `${value.length} ${field} entries, the limit is ${spec.list}.`);
  const checked: unknown[] = [];
  for (const item of value as unknown[]) {
    if (isBlank(item)) return reject('required', `${field} has an empty entry.`);
    const verdict = checkValue(field, spec, item, cx);
    if (!verdict.ok) return verdict;
    checked.push(verdict.value);
  }
  return { ok: true, value: checked };
};

export { checkList };
