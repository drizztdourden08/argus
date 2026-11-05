/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import { useCallback, useEffect, useState } from 'react';
import { formatFilters, readStored, writeStored } from '../../../shared';
import { DEFAULT_FILTERS, STORE_KEYS, parseStored, shownDirection } from '../model';
import type { ItemGroup, ItemOrder, KanbanFilters, StoredView } from '../model';
import type { KanbanState } from '../kanban.type';

const loadView = (): StoredView =>
  parseStored({
    filters: readStored(STORE_KEYS.filters),
    group: readStored(STORE_KEYS.group),
    direction: readStored(STORE_KEYS.direction),
    showDone: readStored(STORE_KEYS.showDone),
  });

const useKanbanState = (): KanbanState => {
  const [view, setView] = useState<StoredView>(loadView);
  const { filters, prefs } = view;

  useEffect(() => {
    writeStored(STORE_KEYS.filters, formatFilters(readStored(STORE_KEYS.filters), filters, prefs.order));
    writeStored(STORE_KEYS.group, prefs.group);
    writeStored(STORE_KEYS.direction, shownDirection(prefs.order, prefs.direction));
    writeStored(STORE_KEYS.showDone, prefs.showDone ? '1' : '0');
  }, [filters, prefs]);

  const patchFilters = useCallback((patch: Partial<KanbanFilters>) => setView((now) => ({ ...now, filters: { ...now.filters, ...patch } })), []);
  const clearFilters = useCallback(() => setView((now) => ({ ...now, filters: DEFAULT_FILTERS })), []);
  const setGroup = useCallback((group: ItemGroup) => setView((now) => ({ ...now, prefs: { ...now.prefs, group } })), []);
  const setOrder = useCallback((order: ItemOrder) => setView((now) => ({ ...now, prefs: { ...now.prefs, order, direction: null } })), []);
  const setDirection = useCallback((direction: SortDirection) => setView((now) => ({ ...now, prefs: { ...now.prefs, direction } })), []);
  const setShowDone = useCallback((showDone: boolean) => setView((now) => ({ ...now, prefs: { ...now.prefs, showDone } })), []);

  return { filters, prefs, patchFilters, clearFilters, setGroup, setOrder, setDirection, setShowDone };
};

export { useKanbanState };
