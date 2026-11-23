/* @layer renderer-app @kind logic */
import type { PendingSend, TranscriptRow } from '../OverlordChat.type';
import { countOf } from './count-of';

const unsettled = (pending: readonly PendingSend[], rows: readonly TranscriptRow[], held: readonly string[]): PendingSend[] =>
  pending.filter((send) => countOf(send.text, rows, held) <= send.baseline);

export { unsettled };
