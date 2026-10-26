/* @layer renderer-app @kind logic */
import type { DocData } from '../api';
import { postEntry } from '../posts';
import { ownerEntry } from '../posts/owner-entry';
import type { SentOp } from './kanban.type';

const sendOp = async (key: string, fields: DocData): Promise<SentOp> => {
  const entry = ownerEntry(fields);
  const result = await postEntry(key, entry);
  return { id: String(entry.id), result };
};

export { sendOp };
