/* @layer renderer-app @kind logic */
import { apiRequest, queryString } from '../api';
import type { TranscriptPage } from './overlord.type';

const readTranscript = (after: number): Promise<TranscriptPage> =>
  apiRequest<TranscriptPage>(`/api/overlord/transcript${queryString({ after })}`);

export { readTranscript };
