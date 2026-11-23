/* @layer renderer-app @kind logic */
import type { TranscriptEvent } from '../../../data/overlord';
import { APPROVAL_WORDS } from '../OverlordChat.constants';
import type { Piece } from '../OverlordChat.type';

const approvalPiece = (event: TranscriptEvent): Piece[] => {
  const word = APPROVAL_WORDS[event.status ?? ''] ?? event.status ?? '';
  const by = event.decidedBy && event.status !== 'pending' ? ` by ${event.decidedBy}` : '';
  const text = `Asked to run ${event.tool ?? 'a tool'}: ${event.summary ?? ''}, ${word}${by}`;
  return [{ kind: 'approval', id: event.id ?? text, text }];
};

export { approvalPiece };
