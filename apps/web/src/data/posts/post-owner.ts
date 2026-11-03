/* @layer renderer-app @kind logic */
import type { DocData, PostResult } from '../api';
import { ownerEntry } from './owner-entry';
import { postEntry } from './post-entry';

const postOwner = (key: string, fields: DocData): Promise<PostResult> => postEntry(key, ownerEntry(fields));

export { postOwner };
