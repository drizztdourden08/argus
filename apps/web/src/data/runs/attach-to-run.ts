/* @layer renderer-app @kind logic */
import { apiRequest, queryString, scopePath } from '../api';
import type { RunAttachment } from './runs.type';

const attachToRun = (key: string, runId: string, image: Blob, caption?: string): Promise<{ attachment: RunAttachment }> =>
  apiRequest<{ attachment: RunAttachment }>(`${scopePath(key, 'runs', runId, 'attachments')}${queryString({ caption })}`, { method: 'POST', raw: image });

export { attachToRun };
