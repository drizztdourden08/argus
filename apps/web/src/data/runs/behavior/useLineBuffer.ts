/* @layer renderer-app @kind logic */
import { useCallback, useEffect, useRef, useState } from 'react';
import { mergeRunLines } from '../merge-run-lines';
import { RUN_LOG_FLUSH_MS } from '../runs.constants';
import type { LineBuffer, RunLogLine } from '../runs.type';

const useLineBuffer = (limit: number): LineBuffer => {
  const [lines, setLines] = useState<RunLogLine[]>([]);
  const queue = useRef<RunLogLine[]>([]);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const local = useRef(0);
  const drop = useCallback(() => {
    if (timer.current !== null) clearTimeout(timer.current);
    timer.current = null;
  }, []);
  const flush = useCallback(() => {
    timer.current = null;
    const batch = queue.current.splice(0);
    setLines((now) => mergeRunLines(now, batch, limit));
  }, [limit]);
  const push = useCallback((incoming: RunLogLine[]) => {
    queue.current.push(...incoming);
    timer.current ??= setTimeout(flush, RUN_LOG_FLUSH_MS);
  }, [flush]);
  const note = useCallback((text: string) => {
    local.current -= 1;
    push([{ seq: local.current, stream: 'sys', at: '', text }]);
  }, [push]);
  const reset = useCallback(() => {
    drop();
    queue.current = [];
    setLines([]);
  }, [drop]);
  const clear = useCallback(() => {
    reset();
    note('view cleared; the log keeps everything');
  }, [reset, note]);
  useEffect(() => drop, [drop]);
  return { lines, push, note, reset, clear };
};

export { useLineBuffer };
