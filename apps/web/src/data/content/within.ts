/* @layer renderer-app @kind logic */
import type { Rejected } from './content.type';
import { reject } from './reject';

const within = (field: string, text: string, max: number | undefined): Rejected | null =>
  max !== undefined && text.length > max ? reject('too-long', `${field} is ${text.length} chars, the limit is ${max}. Shorten it.`) : null;

export { within };
