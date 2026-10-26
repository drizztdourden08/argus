/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { DocLookup, WithId } from '../react.type';
import { useCollectionState } from './useCollectionState';

const useDoc = <T extends object>(scope: string, coll: string, id: string): DocLookup<T> => {
  const state = useCollectionState(scope, coll);
  return useMemo(() => {
    const data = state.docs.get(id);
    return { ready: state.ready, doc: data ? ({ id, ...data } as WithId<T>) : null };
  }, [state, id]);
};

export { useDoc };
