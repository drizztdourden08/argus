/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { BoardMeta, Thread } from '../../types';
import type { BoardView } from '../react.type';
import { useCollection } from './useCollection';
import { useDoc } from './useDoc';
import { useStreamStatus } from './useStreamStatus';

const isLive = (thread: Thread): boolean => !thread.deletedAt && !thread.mergedInto;

const useBoard = (key: string): BoardView => {
  const meta = useDoc<BoardMeta>(key, 'meta', 'board');
  const threads = useCollection<Thread>(key, 'threads');
  const status = useStreamStatus(key);
  const live = useMemo(() => threads.docs.filter(isLive), [threads.docs]);
  return { ready: meta.ready && threads.ready, status, name: meta.doc?.name ?? key, meta: meta.doc, threads: live };
};

export { useBoard };
