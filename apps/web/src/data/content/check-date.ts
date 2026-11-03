/* @layer renderer-app @kind logic */
import type { ItemField } from '../registry';
import { DATE_PATTERN } from './content.constants';
import type { Verdict } from './content.type';
import { onCalendar } from './on-calendar';
import { reject } from './reject';

const checkDate = (field: string, _spec: ItemField, raw: string): Verdict => {
  const text = raw.trim();
  if (!DATE_PATTERN.test(text) || !onCalendar(text)) return reject('bad-date', `${field} is a date like 2026-11-02.`);
  return { ok: true, value: text };
};

export { checkDate };
