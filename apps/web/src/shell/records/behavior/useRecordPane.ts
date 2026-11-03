/* @layer renderer-app @kind logic */
import { useCallback, useMemo } from 'react';
import { useLocation, useNavigate } from 'react-router';
import { OPEN_PARAM, PANE_PARAM, STACK_MAX } from '../records.constants';
import type { RecordPaneState } from '../records.type';
import { parseRecord } from './parse-record';
import { readStack } from './read-stack';
import { recordSearch } from './record-search';
import { stackState } from './stack-state';

const useRecordPane = (): RecordPaneState => {
  const location = useLocation();
  const { search, hash } = location;
  const state: unknown = location.state;
  const navigate = useNavigate();
  const params = new URLSearchParams(search);
  const code = params.get(OPEN_PARAM);
  const tab = params.get(PANE_PARAM);
  const stack = readStack(state);
  const go = useCallback(
    (nextCode: string | null, nextTab: string | null, nextStack: readonly string[]) =>
      void navigate({ search: recordSearch(search, nextCode, nextTab), hash }, { replace: true, state: stackState(nextStack) }),
    [navigate, search, hash],
  );
  const open = useCallback(
    (next: string, nextTab?: string) => {
      const kept = code && code !== next ? [...stack, code].slice(-STACK_MAX) : stack;
      go(next, nextTab ?? null, kept);
    },
    [go, code, stack],
  );
  const step = useCallback((next: string) => go(next, null, stack), [go, stack]);
  const setTab = useCallback((next: string) => go(code, next, stack), [go, code, stack]);
  const back = useCallback(() => go(stack.at(-1) ?? null, null, stack.slice(0, -1)), [go, stack]);
  const close = useCallback(() => go(null, null, []), [go]);
  const ref = useMemo(() => (code ? parseRecord(code) : null), [code]);
  return useMemo(() => ({ ref, tab, canBack: stack.length > 0, open, step, setTab, back, close }), [ref, tab, stack, open, step, setTab, back, close]);
};

export { useRecordPane };
