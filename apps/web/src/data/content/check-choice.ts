/* @layer renderer-app @kind logic */
import type { ItemField } from '../registry';
import type { Verdict } from './content.type';
import { reject } from './reject';

const checkChoice = (field: string, spec: ItemField, raw: string): Verdict => {
  const text = raw.trim();
  const values = spec.values ?? [];
  if (!values.includes(text)) return reject('bad-enum', `${field} is one of ${values.join(', ')}.`);
  return { ok: true, value: text };
};

export { checkChoice };
