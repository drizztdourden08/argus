/* @layer renderer-app @kind logic */
import type { TranscriptLine } from '../../../data/overlord';
import type { Piece, ToolItem, TranscriptRow } from '../OverlordChat.type';
import { piecesOf } from './pieces-of';
import type { Acc, PieceOf, TalkPiece } from './to-rows.type';

const settleTool = (acc: Acc, piece: PieceOf<'result'>): void => {
  const tool = acc.tools.get(piece.id);
  if (!tool) return;
  tool.status = piece.failed ? 'failed' : 'done';
  if (piece.failed && piece.text) tool.summary = `${tool.summary} (error: ${piece.text})`;
};

const placeApproval = (acc: Acc, piece: TalkPiece, key: string): void => {
  const known = acc.approvals.get(piece.id ?? key);
  if (known && known.kind !== 'tools') {
    known.text = piece.text;
    return;
  }
  const row = { ...piece, key };
  acc.approvals.set(piece.id ?? key, row);
  acc.rows.push(row);
};

const placeTool = (acc: Acc, piece: PieceOf<'tool'>, key: string): void => {
  const item: ToolItem = { id: piece.id || key, name: piece.name, summary: piece.summary, status: 'running' };
  acc.tools.set(item.id, item);
  const last = acc.rows.at(-1);
  if (last?.kind === 'tools') last.tools.push(item);
  else acc.rows.push({ kind: 'tools', key, tools: [item] });
};

const place = (acc: Acc, piece: Piece, key: string): void => {
  if (piece.kind === 'result') settleTool(acc, piece);
  else if (piece.kind === 'approval') placeApproval(acc, piece, key);
  else if (piece.kind === 'tool') placeTool(acc, piece, key);
  else acc.rows.push({ ...piece, key });
};

const toRows = (lines: readonly TranscriptLine[]): TranscriptRow[] => {
  const acc: Acc = { rows: [], tools: new Map(), approvals: new Map() };
  lines.forEach((line) => {
    piecesOf(line.event).forEach((piece, at) => place(acc, piece, `${line.index}-${at}`));
  });
  return acc.rows;
};

export { toRows };
