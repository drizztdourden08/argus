/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { TestPingResult } from '../types';

const sendTest = (): Promise<TestPingResult> =>
  apiRequest<TestPingResult>('/api/notify/test', { method: 'POST', json: {} });

export { sendTest };
