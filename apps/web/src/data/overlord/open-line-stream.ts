/* @layer renderer-app @kind logic */
import { queryString } from '../api';
import { STREAM_PATH } from './overlord.constants';
import type { TranscriptLine } from './overlord.type';

const parseLine = (data: string): TranscriptLine | null => {
  try {
    const body = JSON.parse(data) as TranscriptLine;
    return typeof body.index === 'number' ? body : null;
  } catch {
    return null;
  }
};

const openLineStream = (session: string, after: number, onLine: (line: TranscriptLine) => void): (() => void) => {
  const source = new EventSource(`${STREAM_PATH}${queryString({ session, after })}`);
  source.addEventListener('line', (message) => {
    const line = parseLine((message as MessageEvent<string>).data);
    if (line) onLine(line);
  });
  return () => source.close();
};

export { openLineStream };
