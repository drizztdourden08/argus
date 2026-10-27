/* @layer renderer-app @kind logic */
import { useMemo, useState } from 'react';
import { buildView, conflictedTests, followItemThread, inFocus, latestWakes, visibleTests, wakePending } from '../model';
import type { TestsData, TestsScreen, TestsShared } from '../tests.type';
import { useDrawerNav } from './useDrawerNav';
import { useDrawerWide } from './useDrawerWide';
import { useFolds } from './useFolds';
import { useTestsFocus } from './useTestsFocus';
import { useTestAnchor } from './useTestAnchor';
import { useTestNotices } from './useTestNotices';
import { useTestOps } from './useTestOps';
import { useTestsState } from './useTestsState';
import { useWakeClock } from './useWakeClock';

const useTestsScreen = (boardKey: string, data: TestsData): TestsScreen => {
  const state = useTestsState();
  const focus = useTestsFocus(data);
  const notices = useTestNotices(boardKey, data.entries, data.tests);
  const ops = useTestOps(boardKey);
  const folds = useFolds();
  const anchor = useTestAnchor();
  const wide = useDrawerWide();
  const nav = useDrawerNav(anchor.selected, anchor.open);
  const [noteId, setNoteId] = useState<string | null>(null);
  const [editId, setEditId] = useState<string | null>(null);
  const [logId, setLogId] = useState<string | null>(null);
  const now = useWakeClock(wakePending(data.wakes, Date.now()));
  const { filters, prefs } = state;
  const { items, builds, launchers, defs, handed } = data;
  const conflicted = useMemo(() => conflictedTests(notices.notices), [notices.notices]);
  const visible = useMemo(
    () => inFocus(visibleTests(followItemThread(data.tests, items), filters.archived), focus.scope),
    [data.tests, items, filters.archived, focus.scope],
  );
  const view = useMemo(
    () => buildView({ tests: visible, builds, items, launchers, defs, handed, filters, prefs, conflicted }),
    [visible, builds, items, launchers, defs, handed, filters, prefs, conflicted],
  );
  const latest = useMemo(() => latestWakes(data.wakes), [data.wakes]);
  const shared = useMemo<TestsShared>(() => ({
    boardKey,
    ops,
    items,
    threads: data.threads,
    wakes: { all: data.wakes, latest, now },
    group: prefs.group,
    order: prefs.order,
    showPassed: prefs.showPassed,
    conflicted,
    folds,
    onOpen: anchor.open,
    onNote: setNoteId,
    onEdit: setEditId,
    onLog: setLogId,
  }), [boardKey, ops, items, data.threads, data.wakes, latest, now, prefs, conflicted, folds, anchor.open]);
  return {
    state,
    view,
    shared,
    notices,
    visible,
    focus,
    anchor,
    nav,
    wide,
    noteId,
    editId,
    logId,
    closeNote: () => setNoteId(null),
    closeEdit: () => setEditId(null),
    closeLog: () => setLogId(null),
  };
};

export { useTestsScreen };
