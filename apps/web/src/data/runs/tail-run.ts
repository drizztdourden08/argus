/* @layer renderer-app @kind logic */
import type { TabLink } from '../hub/tab-link';
import { LIVE_RUN_STATES } from './runs.constants';
import type { RunTailHandlers } from './runs.type';

const tailRun = (link: TabLink, key: string, runId: string, handlers: RunTailHandlers, after = 0): (() => void) => {
  const release = link.tail(key, runId, after, {
    onLine: handlers.onLine,
    onStatus: (run) => {
      handlers.onStatus?.(run);
      if (!LIVE_RUN_STATES.includes(run.status)) release();
    },
  });
  return release;
};

export { tailRun };
