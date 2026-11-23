/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import { useCallback, useEffect, useState } from 'react';
import { formatFilters, readStored, writeStored } from '../../../shared';
import { DEFAULT_FILTERS, STORE_KEYS, parseFolded, parseStored, shownDirection } from '../model';
import { boardFlagKey } from '../model/board-flag-key';
import type { ItemGroup, ItemOrder, KanbanFilters, StoredView } from '../model';
import type { KanbanState } from '../kanban.type';

const storedFlag = (base: string, boardKey: string): string | null => readStored(boardFlagKey(base, boardKey)) ?? readStored(base);

const loadView = (boardKey: string): StoredView =>
  parseStored({
    filters: readStored(STORE_KEYS.filters),
    group: readStored(STORE_KEYS.group),
    direction: readStored(STORE_KEYS.direction),
    showDone: readStored(STORE_KEYS.showDone),
    compact: storedFlag(STORE_KEYS.compact, boardKey),
    epicsView: storedFlag(STORE_KEYS.epicsView, boardKey),
  });

const loadFolded = (boardKey: string): readonly string[] => parseFolded(readStored(boardFlagKey(STORE_KEYS.foldedGroups, boardKey)));

const useKanbanState = (boardKey: string): KanbanState => {
  const [held, setHeld] = useState<{ boardKey: string; view: StoredView }>(() => ({ boardKey, view: loadView(boardKey) }));
  const current = held.boardKey === boardKey ? held : { boardKey, view: loadView(boardKey) };
  if (current !== held) setHeld(current);
  const { filters, prefs } = current.view;
  const [heldFolds, setHeldFolds] = useState<{ boardKey: string; groups: readonly string[] }>(() => ({ boardKey, groups: loadFolded(boardKey) }));
  const foldedGroups = heldFolds.boardKey === boardKey ? heldFolds.groups : loadFolded(boardKey);
  if (heldFolds.boardKey !== boardKey) setHeldFolds({ boardKey, groups: foldedGroups });

  useEffect(() => {
    writeStored(STORE_KEYS.filters, formatFilters(readStored(STORE_KEYS.filters), filters, prefs.order));
    writeStored(STORE_KEYS.group, prefs.group);
    writeStored(STORE_KEYS.direction, shownDirection(prefs.order, prefs.direction));
    writeStored(STORE_KEYS.showDone, prefs.showDone ? '1' : '0');
    writeStored(boardFlagKey(STORE_KEYS.compact, boardKey), prefs.compact ? '1' : '0');
    writeStored(boardFlagKey(STORE_KEYS.epicsView, boardKey), prefs.epicsView ? '1' : '0');
  }, [boardKey, filters, prefs]);

  useEffect(() => {
    writeStored(boardFlagKey(STORE_KEYS.foldedGroups, boardKey), JSON.stringify(foldedGroups));
  }, [boardKey, foldedGroups]);

  const setView = useCallback(
    (update: (now: StoredView) => StoredView) => setHeld((now) => ({ boardKey, view: update(now.boardKey === boardKey ? now.view : loadView(boardKey)) })),
    [boardKey],
  );
  const patchFilters = useCallback((patch: Partial<KanbanFilters>) => setView((now) => ({ ...now, filters: { ...now.filters, ...patch } })), [setView]);
  const clearFilters = useCallback(() => setView((now) => ({ ...now, filters: DEFAULT_FILTERS })), [setView]);
  const setGroup = useCallback((group: ItemGroup) => setView((now) => ({ ...now, prefs: { ...now.prefs, group } })), [setView]);
  const setOrder = useCallback((order: ItemOrder) => setView((now) => ({ ...now, prefs: { ...now.prefs, order, direction: null } })), [setView]);
  const setDirection = useCallback((direction: SortDirection) => setView((now) => ({ ...now, prefs: { ...now.prefs, direction } })), [setView]);
  const setShowDone = useCallback((showDone: boolean) => setView((now) => ({ ...now, prefs: { ...now.prefs, showDone } })), [setView]);
  const setCompact = useCallback((compact: boolean) => setView((now) => ({ ...now, prefs: { ...now.prefs, compact } })), [setView]);
  const setEpicsView = useCallback((epicsView: boolean) => setView((now) => ({ ...now, prefs: { ...now.prefs, epicsView } })), [setView]);
  const setFoldedGroups = useCallback((groups: readonly string[]) => setHeldFolds({ boardKey, groups }), [boardKey]);

  return { filters, prefs, patchFilters, clearFilters, setGroup, setOrder, setDirection, setShowDone, compact: prefs.compact, epicsView: prefs.epicsView, setCompact, setEpicsView, foldedGroups, setFoldedGroups };
};

export { useKanbanState };
