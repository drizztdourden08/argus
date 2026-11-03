/* @layer renderer-app @kind logic */
import type { PostResult } from '../api';
import { postOwner } from './post-owner';

const retireLauncher = (key: string, name: string, note: string): Promise<PostResult> =>
  postOwner(key, { kind: 'launcher-retire', target: { c: 'launchers', id: name }, set: note ? { note } : {} });

export { retireLauncher };
