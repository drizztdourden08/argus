/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';

const sendMessage = (text: string): Promise<unknown> => apiRequest('/api/overlord/message', { method: 'POST', json: { text } });

export { sendMessage };
