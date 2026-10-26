/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { CollectionResult, WithId } from '../react.type';
import { useCollectionState } from './useCollectionState';

const useCollection = <T extends object>(scope: string, coll: string): CollectionResult<T> => {
  const state = useCollectionState(scope, coll);
  return useMemo(
    () => ({ ready: state.ready, docs: [...state.docs].map(([id, data]) => ({ id, ...data }) as WithId<T>) }),
    [state],
  );
};

export { useCollection };
