/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { Entry, ThreadChoice } from '../../../../data/kanban';
import { useCollection } from '../../../../data/react';
import { useStored, useStoredFlag } from '../../shared';
import { focusChoices, pausedThreads, STORE_KEYS } from '../../tabs/kanban';
import type { FocusView } from '../BoardFocus.type';

const useFocusChoices = (boardKey: string): FocusView => {
  const threads = useCollection<ThreadChoice>(boardKey, 'threads');
  const entries = useCollection<Entry>(boardKey, 'entries');
  const [focus, setFocus] = useStored(STORE_KEYS.focus);
  const [on, set] = useStoredFlag(STORE_KEYS.showPaused);
  const choices = useMemo(
    () => focusChoices(threads.docs, pausedThreads(threads.docs, entries.docs)),
    [threads.docs, entries.docs],
  );
  return { focus: { focus, setFocus }, showPaused: { on, set }, choices };
};

export { useFocusChoices };
