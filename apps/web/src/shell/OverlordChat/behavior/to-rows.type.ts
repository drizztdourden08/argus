/* @layer renderer-app @kind type */
import type { Piece, ToolItem, TalkRow, TranscriptRow } from '../OverlordChat.type';

interface Acc {
  rows: TranscriptRow[];
  tools: Map<string, ToolItem>;
  approvals: Map<string, TranscriptRow>;
}

type PieceOf<K extends Piece['kind']> = Extract<Piece, { kind: K }>;

type TalkPiece = Omit<TalkRow, 'key'>;

export type { Acc, PieceOf, TalkPiece };
