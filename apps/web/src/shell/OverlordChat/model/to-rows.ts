/* @layer renderer-app @kind logic */
import type { TranscriptLine } from '../../../data/overlord';
import type { TranscriptRow } from './model.type';
import { piecesOf } from './pieces-of';

const toRows = (lines: readonly TranscriptLine[]): TranscriptRow[] => {
  const rows: TranscriptRow[] = [];
  lines.forEach((line) => {
    piecesOf(line.event).forEach((piece, at) => {
      const key = `${line.index}-${at}`;
      const last = rows.at(-1);
      if (piece.kind !== 'tool') return void rows.push({ ...piece, key });
      if (last?.kind === 'tools') return void last.names.push(piece.name);
      rows.push({ kind: 'tools', key, names: [piece.name] });
    });
  });
  return rows;
};

export { toRows };
