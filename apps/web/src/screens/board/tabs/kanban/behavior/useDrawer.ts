/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { STORE_KEYS, neighbour } from '../model';
import type { KanbanFilters, KanbanView } from '../model';
import type { DrawerState, KanbanData } from '../kanban.type';
import { useOpenItem } from './useOpenItem';
import { usePagerKeys } from './usePagerKeys';
import { useStoredFlag } from './useStoredFlag';

const useDrawer = (data: KanbanData, view: KanbanView, filters: KanbanFilters): DrawerState => {
  const target = useOpenItem(data.actions);
  const [wide, setWide] = useStoredFlag(STORE_KEYS.wide);
  const opened =
    data.items.find((item) => item.id === target.id && !item.deletedAt && (filters.archived || !item.archivedAt)) ?? null;
  const ids = useMemo(() => view.columns.flatMap((column) => column.cards.map((card) => card.id)), [view.columns]);
  const previous = opened ? neighbour(ids, opened.id, -1) : null;
  const next = opened ? neighbour(ids, opened.id, 1) : null;
  usePagerKeys(previous, next, target.open);
  return { opened, nav: { previous, next, onStep: target.open }, wide, onWide: () => setWide(!wide), open: target.open, close: target.close };
};

export { useDrawer };
