/* @layer renderer-app @kind logic */
import type { RecordRef } from '../records.type';

const boardCode = (ref: RecordRef): string => (ref.board ? `${ref.board}/${ref.code}` : ref.code);

export { boardCode };
