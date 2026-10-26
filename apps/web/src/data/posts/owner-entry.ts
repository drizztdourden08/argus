/* @layer renderer-app @kind logic */
import type { DocData } from '../api';

const entryId = (): string => {
  const tail = Math.random().toString(36).slice(2, 6).padEnd(4, '0');
  return `O-${Date.now().toString(36)}-${tail}`;
};

const ownerEntry = (fields: DocData): DocData => ({
  id: entryId(),
  author: 'human',
  replyTo: null,
  item: null,
  thread: null,
  status: 'new',
  createdAt: new Date().toISOString(),
  ...fields,
});

export { ownerEntry };
