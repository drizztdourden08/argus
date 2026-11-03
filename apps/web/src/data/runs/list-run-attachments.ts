/* @layer renderer-app @kind logic */
import { apiRequest, scopePath } from '../api';
import type { RunAttachment } from './runs.type';

const listRunAttachments = async (key: string, runId: string): Promise<RunAttachment[]> => {
  const result = await apiRequest<{ attachments?: RunAttachment[] }>(scopePath(key, 'runs', runId, 'attachments'));
  return result.attachments ?? [];
};

export { listRunAttachments };
