/* @layer renderer-app @kind logic */
import type { PostResult } from '../api';
import { postOwner } from './post-owner';

const unlinkLauncher = (key: string, name: string): Promise<PostResult> =>
  postOwner(key, { kind: 'launcher-link', target: { c: 'launchers', id: name }, set: { for: null } });

export { unlinkLauncher };
