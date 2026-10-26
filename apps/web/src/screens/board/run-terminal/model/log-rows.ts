/* @layer renderer-app @kind logic */
import type { LogRow } from '@drizztdourden08/tessera';
import type { RunLogLine } from '../../../../data/runs';

const timeText = (at: string): string => {
  const date = new Date(at);
  return Number.isNaN(date.getTime()) ? '' : date.toLocaleTimeString([], { hour12: false });
};

const rowOf = (line: RunLogLine): LogRow => ({
  id: String(line.seq),
  gutter: timeText(line.at),
  tag: line.stream === 'out' ? '' : line.stream,
  kind: line.stream,
  message: line.text.replace(/\n$/, ''),
});

const logRows = (lines: readonly RunLogLine[]): LogRow[] => lines.map(rowOf);

export { logRows };
