/* @layer renderer-app @kind logic */
import { useCallback, useEffect, useState } from 'react';
import { readStored } from '../../kanban/behavior/read-stored';
import { writeStored } from '../../kanban/behavior/write-stored';
import { formatFilters } from '../../kanban/model';
import { DEFAULT_FILTERS, parseStored, STORE_KEYS } from '../model';
import type { StoredView, TestFilters, TestPrefs } from '../model';
import type { TestsState } from '../tests.type';

const loadView = (): StoredView =>
  parseStored({
    filters: readStored(STORE_KEYS.filters),
    group: readStored(STORE_KEYS.group),
    passed: readStored(STORE_KEYS.passed),
    closed: readStored(STORE_KEYS.closed),
  });

const saveView = ({ filters, prefs }: StoredView): void => {
  writeStored(STORE_KEYS.filters, formatFilters(readStored(STORE_KEYS.filters), filters, prefs.order));
  writeStored(STORE_KEYS.group, prefs.group);
  writeStored(STORE_KEYS.passed, prefs.showPassed ? '1' : '0');
  writeStored(STORE_KEYS.closed, prefs.showClosed ? '1' : '0');
};

const useTestsState = (): TestsState => {
  const [view, setView] = useState<StoredView>(loadView);
  useEffect(() => saveView(view), [view]);
  const patchFilters = useCallback((patch: Partial<TestFilters>) => setView((now) => ({ ...now, filters: { ...now.filters, ...patch } })), []);
  const patchPrefs = useCallback((patch: Partial<TestPrefs>) => setView((now) => ({ ...now, prefs: { ...now.prefs, ...patch } })), []);
  const clearFilters = useCallback(() => setView((now) => ({ ...now, filters: DEFAULT_FILTERS })), []);
  return {
    filters: view.filters,
    prefs: view.prefs,
    patchFilters,
    clearFilters,
    setGroup: (group) => patchPrefs({ group }),
    setOrder: (order) => patchPrefs({ order }),
    setShowPassed: (showPassed) => patchPrefs({ showPassed }),
    setShowClosed: (showClosed) => patchPrefs({ showClosed }),
  };
};

export { useTestsState };
