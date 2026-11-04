/* @layer renderer-app @kind logic */
import { useCallback, useContext, useSyncExternalStore } from 'react';
import { EMPTY_COLLECTION } from '../../stream';
import { DataContext } from '../data-context';
import { NO_UNSUBSCRIBE } from './use-optional-doc.constants';

const useOptionalDoc = <T extends object>(scope: string | null, coll: string, id: string): T | null => {
  const streams = useContext(DataContext);
  const stream = scope !== null && streams ? streams.get(scope) : null;
  const subscribe = useCallback((listener: () => void) => (stream ? stream.subscribe(coll, listener) : NO_UNSUBSCRIBE), [stream, coll]);
  const state = useSyncExternalStore(subscribe, () => (stream ? stream.getCollection(coll) : EMPTY_COLLECTION));
  return (state.docs.get(id) as T | undefined) ?? null;
};

export { useOptionalDoc };
