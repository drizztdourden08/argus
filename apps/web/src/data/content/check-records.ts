/* @layer renderer-app @kind logic */
import type { Verdict } from './content.type';
import { reject } from './reject';

const isRecord = (item: unknown): boolean => typeof item === 'object' && item !== null && !Array.isArray(item);

const checkRecords = (field: string, value: unknown): Verdict => {
  if (!Array.isArray(value)) return reject('not-a-list', `${field} must be a list.`);
  if (!value.every(isRecord)) return reject('bad-record', `${field} holds records (objects).`);
  return { ok: true, value };
};

export { checkRecords };
