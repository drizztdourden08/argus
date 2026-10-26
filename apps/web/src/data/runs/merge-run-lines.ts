/* @layer renderer-app @kind logic */
import type { RunLogLine } from './runs.type';

const newestSeq = (lines: readonly RunLogLine[]): number => lines.reduce((top, line) => Math.max(top, line.seq), 0);

const mergeRunLines = (current: readonly RunLogLine[], incoming: readonly RunLogLine[], cap: number): RunLogLine[] => {
  let cursor = newestSeq(current);
  const added: RunLogLine[] = [];
  for (const line of incoming) {
    if (line.seq > 0 && line.seq <= cursor) continue;
    added.push(line);
    cursor = Math.max(cursor, line.seq);
  }
  return [...current, ...added].slice(-cap);
};

export { mergeRunLines };
