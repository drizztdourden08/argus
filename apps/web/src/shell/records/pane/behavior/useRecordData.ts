/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection } from '../../../../data/react';
import type { Entry } from '../../../../data/types';
import { useFeedActions } from '../../../../screens/board/OwnerFeed';
import { useItemOps } from '../../../../screens/board/tabs/kanban/behavior/useItemOps';
import { useItemRecord } from '../../../../screens/board/tabs/kanban/behavior/useItemRecord';
import { useKanbanData } from '../../../../screens/board/tabs/kanban/behavior/useKanbanData';
import { useNow } from '../../../../screens/board/tabs/kanban/behavior/useNow';
import { useRecordActions } from '../../../../screens/board/tabs/kanban/behavior/useRecordActions';
import { useRecordPane } from '../../behavior/useRecordPane';
import type { RecordRef } from '../../records.type';
import { buildRecordData } from '../model/build-record-data';
import { EXTRA_STORES, FALLBACK_STORE } from '../model/pane-model.constants';
import type { PaneDoc, RecordLoad } from '../pane.type';
import { useRemoteDocs } from './useRemoteDocs';

const useRecordData = (boardKey: string, target: RecordRef): RecordLoad => {
  const kanban = useKanbanData(boardKey);
  const other = useCollection<PaneDoc>(boardKey, EXTRA_STORES[target.collection] ?? FALLBACK_STORE);
  const remote = useRemoteDocs(boardKey, target.collection);
  const entries = useCollection<Entry>(boardKey, 'entries');
  const record = useItemRecord(boardKey, target.id, kanban.actions);
  const ops = useItemOps(boardKey);
  const acts = useRecordActions(boardKey);
  const feed = useFeedActions(boardKey);
  const now = useNow();
  const { open } = useRecordPane();
  const docs = remote.remote ? remote.docs : other.docs;
  const data = useMemo(
    () => buildRecordData({ boardKey, target, kanban, other: docs, entries: entries.docs, record, now, ops, acts, feed, open }),
    [boardKey, target, kanban, docs, entries.docs, record, now, ops, acts, feed, open],
  );
  return { data, ready: kanban.ready && (remote.remote ? remote.ready : other.ready) };
};

export { useRecordData };
