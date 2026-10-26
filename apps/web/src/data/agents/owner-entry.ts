/* @layer renderer-app @kind logic */
import type { DocData } from '../api';
import { ID_SUFFIX_LENGTH } from './agents.constants';

const newEntryId = (): string =>
  `O-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 2 + ID_SUFFIX_LENGTH).padEnd(ID_SUFFIX_LENGTH, '0')}`;

const ownerEntry = (fields: DocData): DocData => ({
  id: newEntryId(),
  author: 'human',
  replyTo: null,
  item: null,
  thread: null,
  status: 'new',
  createdAt: new Date().toISOString(),
  ...fields,
});

export { ownerEntry };
