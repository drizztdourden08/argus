/* @layer renderer-app @kind logic */
import { useEffect, useRef } from 'react';
import { errorMessage } from '../../api';
import { fetchRunLogs } from '../fetch-run-logs';
import type { RunLogOptions, RunLogView } from '../runs.type';
import { tailRun } from '../tail-run';
import { useLineBuffer } from './useLineBuffer';

const useRunLog = (key: string, runId: string | null, options: RunLogOptions): RunLogView => {
  const { limit, live, enabled } = options;
  const { lines, push, note, reset, clear } = useLineBuffer(limit);
  const liveNow = useRef(live);
  useEffect(() => {
    liveNow.current = live;
  }, [live]);
  useEffect(() => {
    if (!enabled || runId === null) return undefined;
    reset();
    let cancelled = false;
    let stop: (() => void) | null = null;
    fetchRunLogs(key, runId, { tail: limit }).then((page) => {
      if (cancelled) return;
      push(page.lines ?? []);
      if ((page.dropped ?? 0) > 0) note(`... ${page.dropped} older lines were dropped to keep the log small`);
      if (liveNow.current) stop = tailRun(key, runId, { onLine: (line) => push([line]) }, page.next);
    }, (error: unknown) => {
      if (!cancelled) note(`could not load the output: ${errorMessage(error)}`);
    });
    return () => {
      cancelled = true;
      stop?.();
    };
  }, [key, runId, enabled, limit, push, note, reset]);
  return { lines, clear };
};

export { useRunLog };
