/* @layer renderer-app @kind constants */
import type { SegmentOption } from '@drizztdourden08/tessera';
import type { SendKind } from './model';

const SENT_LIMIT = 40;

const NOTHING_SENT = 'Nothing sent down yet.';

const SEND_MESSAGE = 'It reaches the lead at once; a lead that is asleep reads it when it wakes.';

const SEND_KINDS: SegmentOption<SendKind>[] = [
  { value: 'decision', label: 'Decision' },
  { value: 'followup', label: 'Note' },
];

const KIND_WORD: Readonly<Record<string, string>> = { decision: 'decision', followup: 'note' };

export { KIND_WORD, NOTHING_SENT, SEND_KINDS, SEND_MESSAGE, SENT_LIMIT };
