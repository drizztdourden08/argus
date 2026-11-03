/* @layer renderer-app @kind logic */
import type { RecordRef } from '../records.type';

const ownScope = (stored: string | null, ref: RecordRef | null, board: string | undefined, here: string | null): string | null =>
  stored !== null && (ref?.board ?? board ?? here) === here ? here : null;

export { ownScope };
