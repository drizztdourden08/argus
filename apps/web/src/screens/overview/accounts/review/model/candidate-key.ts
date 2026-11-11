/* @layer renderer-app @kind logic */
import type { CleanupCandidate } from '../../../../../data/types';

const candidateKey = (candidate: CleanupCandidate): string => `${candidate.account}|${candidate.sessionId}`;

export { candidateKey };
