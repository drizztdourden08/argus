/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { readBoardReleases, readChecks, useRemoteRead } from '../../../../data/remote';
import { checkDocs } from '../model/check-docs';
import { releaseDocs } from '../model/release-docs';
import type { RemoteDocs, RemoteLoader } from '../pane.type';

const NONE: RemoteDocs['docs'] = [];

const LOADERS: Readonly<Record<string, RemoteLoader>> = {
  releases: async (boardKey) => releaseDocs(await readBoardReleases(boardKey)),
  checks: async (boardKey) => checkDocs(await readChecks(boardKey)),
};

const useRemoteDocs = (boardKey: string, collection: string): RemoteDocs => {
  const remote = Object.hasOwn(LOADERS, collection);
  const load = useCallback(async () => LOADERS[collection]?.(boardKey) ?? NONE, [boardKey, collection]);
  const { data, error } = useRemoteRead(load);
  return { remote, docs: data ?? NONE, ready: !remote || data !== null || error !== '' };
};

export { useRemoteDocs };
