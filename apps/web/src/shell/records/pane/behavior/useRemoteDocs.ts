/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { useRemoteRead } from '../../../../data/remote';
import type { RemoteDocs } from '../pane.type';
import { LOADERS, NONE } from './use-remote-docs.constants';

const useRemoteDocs = (boardKey: string, collection: string): RemoteDocs => {
  const remote = Object.hasOwn(LOADERS, collection);
  const load = useCallback(async () => LOADERS[collection]?.(boardKey) ?? NONE, [boardKey, collection]);
  const { data, error } = useRemoteRead(load);
  return { remote, docs: data ?? NONE, ready: !remote || data !== null || error !== '' };
};

export { useRemoteDocs };
