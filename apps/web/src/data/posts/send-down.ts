/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { PostResult } from '../api';
import type { SendDownInput } from './posts.type';

const sendDown = (input: SendDownInput): Promise<PostResult> =>
  apiRequest<PostResult>('/api/portfolio/send', { method: 'POST', json: input });

export { sendDown };
