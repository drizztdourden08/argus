/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { focusKey, useStored, useStoredFlag } from '../../../shared';
import { pausedThreads } from '../../kanban/model';
import { focusOptions, STORE_KEYS } from '../model';
import type { TestsData, TestsFocus } from '../tests.type';

const useTestsFocus = (boardKey: string, data: TestsData): TestsFocus => {
  const [stored, setFocus] = useStored(focusKey(boardKey));
  const [showPaused, setShowPaused] = useStoredFlag(STORE_KEYS.paused);
  const paused = useMemo(() => pausedThreads(data.threads, data.entries), [data.threads, data.entries]);
  const options = useMemo(() => focusOptions(data.threads, paused), [data.threads, paused]);
  const known = data.threads.some((thread) => thread.id === stored);
  return {
    scope: { focus: known ? stored : '', paused, showPaused },
    options,
    pausedCount: options.filter((option) => option.paused).length,
    setFocus,
    setShowPaused,
  };
};

export { useTestsFocus };
