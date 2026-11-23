/* @layer renderer-app @kind logic */
import type { ChatTranscriptMessage } from '@drizztdourden08/tessera';
import { QUEUED_NOTE, SENDING_NOTE, SENT_NOTE, TALK_ROLE, TALK_WHO } from '../OverlordChat.constants';
import type { PendingSend, TranscriptRow } from '../OverlordChat.type';

const countOf = (text: string, rows: readonly TranscriptRow[], held: readonly string[]): number => {
  const wanted = text.trim();
  const said = rows.filter((row) => (row.kind === 'owner' || row.kind === 'wake') && row.text.trim() === wanted).length;
  return said + held.filter((one) => one.trim() === wanted).length;
};

const unsettled = (pending: readonly PendingSend[], rows: readonly TranscriptRow[], held: readonly string[]): PendingSend[] =>
  pending.filter((send) => countOf(send.text, rows, held) <= send.baseline);

const ownerBubble = (id: string, text: string, note: string): ChatTranscriptMessage => ({
  id,
  speaker: TALK_WHO.owner,
  role: TALK_ROLE.owner,
  content: text,
  time: note,
});

const queuedMessages = (held: readonly string[], pending: readonly PendingSend[]): ChatTranscriptMessage[] => [
  ...held.map((text, at) => ownerBubble(`held-${at}`, text, QUEUED_NOTE)),
  ...pending.map((send) => ownerBubble(`pending-${send.id}`, send.text, send.state === 'sending' ? SENDING_NOTE : SENT_NOTE)),
];

export { countOf, queuedMessages, unsettled };
