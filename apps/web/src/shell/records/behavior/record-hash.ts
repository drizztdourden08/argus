/* @layer renderer-app @kind logic */
import type { RecordRef } from '../records.type';

const recordHash = (ref: RecordRef): string => (ref.tab ? `${ref.id}/${ref.tab}` : ref.id);

export { recordHash };
