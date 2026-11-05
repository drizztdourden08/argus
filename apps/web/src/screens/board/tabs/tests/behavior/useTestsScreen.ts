/* @layer renderer-app @kind logic */
import { useMemo, useState } from 'react';
import { useRecordOrder, useRecordPane } from '../../../../../shell';
import { conflictedTests, followItemThread, inFocus, latestWakes, shownIds, testsView, visibleTests, wakePending } from '../model';
import type { TestsData, TestsScreen, TestsShared } from '../tests.type';
import { useFolds } from './useFolds';
import { useTestsFocus } from './useTestsFocus';
import { useTestNotices } from './useTestNotices';
import { useTestOps } from './useTestOps';
import { useTestsState } from './useTestsState';
import { useWakeClock } from './useWakeClock';

const useTestsScreen = (boardKey: string, data: TestsData): TestsScreen => {
  const state = useTestsState();
  const focus = useTestsFocus(boardKey, data);
  const notices = useTestNotices(boardKey, data.entries, data.tests);
  const ops = useTestOps(boardKey);
  const folds = useFolds();
  const pane = useRecordPane();
  const [noteId, setNoteId] = useState<string | null>(null);
  const [editId, setEditId] = useState<string | null>(null);
  const [logId, setLogId] = useState<string | null>(null);
  const now = useWakeClock(wakePending(data.wakes, Date.now()));
  const { filters, prefs } = state;
  const { items, launchers, defs } = data;
  const conflicted = useMemo(() => conflictedTests(notices.notices), [notices.notices]);
  const visible = useMemo(
    () => inFocus(visibleTests(followItemThread(data.tests, items), filters.archived), focus.scope),
    [data.tests, items, filters.archived, focus.scope],
  );
  const view = useMemo(
    () => testsView({ tests: visible, items, launchers, defs, filters, prefs, conflicted }),
    [visible, items, launchers, defs, filters, prefs, conflicted],
  );
  useRecordOrder(useMemo(() => shownIds(view, prefs.group), [view, prefs.group]));
  const latest = useMemo(() => latestWakes(data.wakes), [data.wakes]);
  const shared = useMemo<TestsShared>(() => ({
    boardKey,
    ops,
    items,
    threads: data.threads,
    wakes: { all: data.wakes, latest, now },
    group: prefs.group,
    order: prefs.order,
    direction: prefs.direction,
    showPassed: prefs.showPassed,
    layout: state.layout,
    conflicted,
    folds,
    onOpen: pane.open,
    onNote: setNoteId,
    onEdit: setEditId,
    onLog: setLogId,
  }), [boardKey, ops, items, data.threads, data.wakes, latest, now, prefs, state.layout, conflicted, folds, pane.open]);
  return {
    state,
    view,
    shared,
    notices,
    visible,
    focus,
    noteId,
    editId,
    logId,
    closeNote: () => setNoteId(null),
    closeEdit: () => setEditId(null),
    closeLog: () => setLogId(null),
  };
};

export { useTestsScreen };
