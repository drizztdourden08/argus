/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { buildView, focusChoices, pausedThreads } from '../model';
import type { KanbanViewInput, KanbanViewResult } from '../kanban.type';

const useKanbanView = (input: KanbanViewInput): KanbanViewResult => {
  const { data, state, focus, notices, now } = input;
  const { items, threads, entries, needs } = data;
  const paused = useMemo(() => pausedThreads(threads, entries), [threads, entries]);
  const choices = useMemo(() => focusChoices(threads, paused), [threads, paused]);
  const view = useMemo(
    () => buildView({
      items, threads, needs, notices: notices.notices, filters: state.filters, prefs: state.prefs, paused, focus: focus.focus, showPaused: focus.showPaused, now,
    }),
    [items, threads, needs, notices.notices, state.filters, state.prefs, paused, focus.focus, focus.showPaused, now],
  );
  return { view, choices };
};

export { useKanbanView };
