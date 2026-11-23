/* @layer renderer-app @kind logic */
import type { TranscriptRow } from '../OverlordChat.type';

const countOf = (text: string, rows: readonly TranscriptRow[], held: readonly string[]): number => {
  const wanted = text.trim();
  const said = rows.filter((row) => (row.kind === 'owner' || row.kind === 'wake') && row.text.trim() === wanted).length;
  return said + held.filter((one) => one.trim() === wanted).length;
};

export { countOf };
