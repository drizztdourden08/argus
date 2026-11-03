/* @layer renderer-app @kind logic */
import { useCallback, useMemo } from 'react';
import { useLocation, useNavigate, useParams } from 'react-router';
import { OPEN_PARAM, PANE_PARAM, STACK_MAX } from '../records.constants';
import type { RecordPaneState } from '../records.type';
import { parseRecord } from './parse-record';
import { readStack } from './read-stack';
import { recordSearch } from './record-search';
import { stackState } from './stack-state';
import { stickyCode } from './sticky-code';

const useRecordPane = (): RecordPaneState => {
  const location = useLocation();
  const { search, hash } = location;
  const state: unknown = location.state;
  const here = useParams().key ?? null;
  const navigate = useNavigate();
  const params = new URLSearchParams(search);
  const code = params.get(OPEN_PARAM);
  const tab = params.get(PANE_PARAM);
  const stack = readStack(state);
  const ref = useMemo(() => (code ? parseRecord(code) : null), [code]);
  const go = useCallback(
    (nextCode: string | null, nextTab: string | null, nextStack: readonly string[]) =>
      void navigate({ search: recordSearch(search, nextCode, nextTab), hash }, { replace: true, state: stackState(nextStack) }),
    [navigate, search, hash],
  );
  const open = useCallback(
    (next: string, nextTab?: string) => {
      const target = stickyCode(next, here, ref);
      const kept = code && code !== target ? [...stack, code].slice(-STACK_MAX) : stack;
      go(target, nextTab ?? null, kept);
    },
    [go, code, stack, here, ref],
  );
  const step = useCallback((next: string) => go(stickyCode(next, here, ref), null, stack), [go, stack, here, ref]);
  const setTab = useCallback((next: string) => go(code, next, stack), [go, code, stack]);
  const back = useCallback(() => go(stack.at(-1) ?? null, null, stack.slice(0, -1)), [go, stack]);
  const close = useCallback(() => go(null, null, []), [go]);
  return useMemo(() => ({ ref, tab, canBack: stack.length > 0, open, step, setTab, back, close }), [ref, tab, stack, open, step, setTab, back, close]);
};

export { useRecordPane };
