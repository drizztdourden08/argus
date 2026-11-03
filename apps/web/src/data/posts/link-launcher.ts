/* @layer renderer-app @kind logic */
import type { PostResult } from '../api';
import { postOwner } from './post-owner';

const linkLauncher = (key: string, name: string, item: string): Promise<PostResult> =>
  postOwner(key, { kind: 'launcher-link', target: { c: 'launchers', id: name }, set: { for: item } });

export { linkLauncher };
