/* @layer renderer-app @kind logic */
import type { RecordRef } from '../records.type';
import { ITEMS } from './use-shown-code.constants';

const storedItemId = (ref: RecordRef | null): string | null => (ref?.collection === ITEMS && ref.code === ref.id ? ref.id : null);

export { storedItemId };
