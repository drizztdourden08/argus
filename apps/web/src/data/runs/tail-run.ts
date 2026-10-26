/* @layer renderer-app @kind logic */
import { queryString, scopePath } from '../api';
import { LIVE_RUN_STATES } from './runs.constants';
import type { RunLogLine, RunRow, RunTailHandlers } from './runs.type';

const parseEvent = <T>(data: unknown): T | null => {
  try {
    return JSON.parse(String(data)) as T;
  } catch {
    return null;
  }
};

const tailRun = (key: string, runId: string, handlers: RunTailHandlers, after = 0): (() => void) => {
  let last = after;
  const source = new EventSource(`${scopePath(key, 'runs', runId, 'logs', 'stream')}${queryString({ after })}`);
  source.addEventListener('log', (event) => {
    const line = parseEvent<RunLogLine>(event.data);
    if (!line || line.seq <= last) return;
    last = line.seq;
    handlers.onLine(line);
  });
  source.addEventListener('status', (event) => {
    const status = parseEvent<{ run: RunRow }>(event.data);
    if (!status) return;
    handlers.onStatus?.(status.run);
    if (!LIVE_RUN_STATES.includes(status.run.status)) source.close();
  });
  return () => source.close();
};

export { tailRun };
