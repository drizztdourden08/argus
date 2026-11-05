/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useBoardItems } from '../../../../data/boards';
import { useNow } from '../../clock';
import type { BoardEntries } from '../boards-view.type';
import { boardRecord } from '../model';
import { useBoardLive } from './useBoardLive';

const useBoardEntries = (): BoardEntries => {
  const { ready, items } = useBoardItems();
  const now = useNow();
  const liveOf = useBoardLive(now);
  const entries = useMemo(
    () => items.map((item) => {
      const record = boardRecord(item, now);
      return { record, card: item.card, live: liveOf(record.key), updatedAt: item.doc.updatedAt ?? '' };
    }),
    [items, now, liveOf],
  );
  return { ready, entries };
};

export { useBoardEntries };
