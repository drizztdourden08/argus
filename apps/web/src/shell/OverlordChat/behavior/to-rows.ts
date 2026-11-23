/* @layer renderer-app @kind logic */
import type { TranscriptLine } from '../../../data/overlord';
import type { ToolItem, TranscriptRow } from '../OverlordChat.type';
import { piecesOf } from './pieces-of';

const toRows = (lines: readonly TranscriptLine[]): TranscriptRow[] => {
  const rows: TranscriptRow[] = [];
  const tools = new Map<string, ToolItem>();
  const approvals = new Map<string, TranscriptRow>();
  lines.forEach((line) => {
    piecesOf(line.event).forEach((piece, at) => {
      const key = `${line.index}-${at}`;
      const last = rows.at(-1);
      if (piece.kind === 'result') {
        const tool = tools.get(piece.id);
        if (!tool) return;
        tool.status = piece.failed ? 'failed' : 'done';
        if (piece.failed && piece.text) tool.summary = `${tool.summary} (error: ${piece.text})`;
        return;
      }
      if (piece.kind === 'approval') {
        const known = approvals.get(piece.id ?? key);
        if (known && known.kind !== 'tools') return void (known.text = piece.text);
        const row = { ...piece, key };
        approvals.set(piece.id ?? key, row);
        return void rows.push(row);
      }
      if (piece.kind !== 'tool') return void rows.push({ ...piece, key });
      const item: ToolItem = { id: piece.id || key, name: piece.name, summary: piece.summary, status: 'running' };
      tools.set(item.id, item);
      if (last?.kind === 'tools') return void last.tools.push(item);
      rows.push({ kind: 'tools', key, tools: [item] });
    });
  });
  return rows;
};

export { toRows };
