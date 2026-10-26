/* @layer renderer-app @kind logic */
import { ApiError } from '../../../../data/api';
import { DOWN_SENTENCE } from '../../overview.constants';
import type { KillSentences } from './model.type';

const killError = (error: unknown, said: KillSentences): string => {
  if (!(error instanceof ApiError)) return `Kill failed: ${error instanceof Error ? error.message : 'unknown'}.`;
  if (error.status === 0) return DOWN_SENTENCE;
  if (error.status === 403) return said.denied;
  if (error.status === 404) return said.gone;
  if (error.status === 409) return said.changed;
  return `Kill failed: ${error.message || error.code || 'unknown'}.`;
};

export { killError };
