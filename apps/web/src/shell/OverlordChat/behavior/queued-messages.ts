/* @layer renderer-app @kind logic */
import type { ChatTranscriptMessage } from '@drizztdourden08/tessera';
import { QUEUED_NOTE, SENDING_NOTE, SENT_NOTE, TALK_ROLE, TALK_WHO } from '../OverlordChat.constants';
import type { PendingSend } from '../OverlordChat.type';

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

export { queuedMessages };
