/* @layer renderer-app @kind logic */
import { useCallback, useSyncExternalStore } from 'react';
import type { CollectionState } from '../../stream';
import { useStreams } from './useStreams';

const useCollectionState = (scope: string, coll: string): CollectionState => {
  const stream = useStreams().get(scope);
  const subscribe = useCallback((listener: () => void) => stream.subscribe(coll, listener), [stream, coll]);
  return useSyncExternalStore(subscribe, () => stream.getCollection(coll));
};

export { useCollectionState };
