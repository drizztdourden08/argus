/* @layer renderer-app @kind logic */
import { apiRequest, scopePath } from '../api';
import type { ReleaseChannel } from './remote.type';

const readBoardReleases = async (key: string): Promise<ReleaseChannel[]> =>
  (await apiRequest<{ channels: ReleaseChannel[] }>(scopePath(key, 'releases'))).channels;

export { readBoardReleases };
