/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { buildView, pausedThreads } from '../model';
import type { KanbanViewInput, KanbanViewResult } from '../kanban.type';

const useKanbanView = (input: KanbanViewInput): KanbanViewResult => {
  const { data, state, focus, notices, now } = input;
  const { items, threads, entries, needs, asks, signals, trails, config } = data;
  const paused = useMemo(() => pausedThreads(threads, entries), [threads, entries]);
  const view = useMemo(
    () => buildView({
      items, threads, needs, asks, signals, trails, notices: notices.notices, filters: state.filters, prefs: state.prefs, paused, focus: focus.focus, showPaused: focus.showPaused, config, now,
    }),
    [items, threads, needs, asks, signals, trails, notices.notices, state.filters, state.prefs, paused, focus.focus, focus.showPaused, config, now],
  );
  return { view };
};

export { useKanbanView };
