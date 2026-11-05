/* @layer renderer-app @kind logic */
import { useCallback, useEffect, useState } from 'react';
import { formatFilters, readStored, writeStored } from '../../../shared';
import { DEFAULT_FILTERS, parseStored, shownDirection, STORE_KEYS } from '../model';
import type { StoredView, TestFilters, TestPrefs } from '../model';
import type { TestsState } from '../tests.type';
import { useCaseLayout } from './useCaseLayout';

const loadView = (): StoredView =>
  parseStored({
    filters: readStored(STORE_KEYS.filters),
    group: readStored(STORE_KEYS.group),
    direction: readStored(STORE_KEYS.direction),
    passed: readStored(STORE_KEYS.passed),
    closed: readStored(STORE_KEYS.closed),
  });

const saveView = ({ filters, prefs }: StoredView): void => {
  writeStored(STORE_KEYS.filters, formatFilters(readStored(STORE_KEYS.filters), filters, prefs.order));
  writeStored(STORE_KEYS.group, prefs.group);
  writeStored(STORE_KEYS.direction, shownDirection(prefs.order, prefs.direction));
  writeStored(STORE_KEYS.passed, prefs.showPassed ? '1' : '0');
  writeStored(STORE_KEYS.closed, prefs.showClosed ? '1' : '0');
};

const useTestsState = (): TestsState => {
  const [view, setView] = useState<StoredView>(loadView);
  const [layout, setLayout] = useCaseLayout();
  useEffect(() => saveView(view), [view]);
  const patchFilters = useCallback((patch: Partial<TestFilters>) => setView((now) => ({ ...now, filters: { ...now.filters, ...patch } })), []);
  const patchPrefs = useCallback((patch: Partial<TestPrefs>) => setView((now) => ({ ...now, prefs: { ...now.prefs, ...patch } })), []);
  const clearFilters = useCallback(() => setView((now) => ({ ...now, filters: DEFAULT_FILTERS })), []);
  return {
    filters: view.filters,
    prefs: view.prefs,
    layout,
    setLayout,
    patchFilters,
    clearFilters,
    setGroup: (group) => patchPrefs({ group }),
    setOrder: (order) => patchPrefs({ order, direction: null }),
    setDirection: (direction) => patchPrefs({ direction }),
    setShowPassed: (showPassed) => patchPrefs({ showPassed }),
    setShowClosed: (showClosed) => patchPrefs({ showClosed }),
  };
};

export { useTestsState };
