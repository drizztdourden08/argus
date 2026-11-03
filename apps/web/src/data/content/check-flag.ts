/* @layer renderer-app @kind logic */
import type { Verdict } from './content.type';
import { reject } from './reject';

const checkFlag = (field: string, value: unknown): Verdict => {
  const word = typeof value === 'string' ? value.trim() : value;
  if (word === true || word === 'true') return { ok: true, value: true };
  if (word === false || word === 'false') return { ok: true, value: false };
  return reject('bad-flag', `${field} is true or false.`);
};

export { checkFlag };
