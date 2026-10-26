/* @layer renderer-app @kind logic */
import type { DocData } from '../api';
import { newEntryId } from './new-entry-id';

const ownerEntry = (fields: DocData, now: Date = new Date()): DocData => ({
  id: newEntryId(now.getTime()),
  author: 'human',
  replyTo: null,
  item: null,
  thread: null,
  status: 'new',
  createdAt: now.toISOString(),
  ...fields,
});

export { ownerEntry };
