/* @layer renderer-app @kind logic */
import { useCallback, useEffect, useState } from 'react';
import { DEFAULT_FILTERS, STORE_KEYS, formatFilters, parseStored } from '../model';
import type { ItemGroup, ItemOrder, KanbanFilters, StoredView } from '../model';
import type { KanbanState } from '../kanban.type';
import { readStored } from './read-stored';
import { writeStored } from './write-stored';

const loadView = (): StoredView =>
  parseStored({ filters: readStored(STORE_KEYS.filters), group: readStored(STORE_KEYS.group), showDone: readStored(STORE_KEYS.showDone) });

const useKanbanState = (): KanbanState => {
  const [view, setView] = useState<StoredView>(loadView);
  const { filters, prefs } = view;

  useEffect(() => {
    writeStored(STORE_KEYS.filters, formatFilters(readStored(STORE_KEYS.filters), filters, prefs.order));
    writeStored(STORE_KEYS.group, prefs.group);
    writeStored(STORE_KEYS.showDone, prefs.showDone ? '1' : '0');
  }, [filters, prefs]);

  const patchFilters = useCallback((patch: Partial<KanbanFilters>) => setView((now) => ({ ...now, filters: { ...now.filters, ...patch } })), []);
  const clearFilters = useCallback(() => setView((now) => ({ ...now, filters: DEFAULT_FILTERS })), []);
  const setGroup = useCallback((group: ItemGroup) => setView((now) => ({ ...now, prefs: { ...now.prefs, group } })), []);
  const setOrder = useCallback((order: ItemOrder) => setView((now) => ({ ...now, prefs: { ...now.prefs, order } })), []);
  const setShowDone = useCallback((showDone: boolean) => setView((now) => ({ ...now, prefs: { ...now.prefs, showDone } })), []);

  return { filters, prefs, patchFilters, clearFilters, setGroup, setOrder, setShowDone };
};

export { useKanbanState };
