/* @layer renderer-app @kind constants */
import { readBoardReleases, readChecks } from '../../../../data/remote';
import { readRequests } from '../../../../data/worktrees';
import { checkDocs } from '../model/check-docs';
import { releaseDocs } from '../model/release-docs';
import { requestDocs } from '../model/request-docs';
import type { RemoteDocs, RemoteLoader } from '../pane.type';

const NONE: RemoteDocs['docs'] = [];

const LOADERS: Readonly<Record<string, RemoteLoader>> = {
  releases: async (boardKey) => releaseDocs(await readBoardReleases(boardKey)),
  checks: async (boardKey) => checkDocs(await readChecks(boardKey)),
  requests: async (boardKey) => requestDocs(await readRequests(boardKey)),
};

export { LOADERS, NONE };
