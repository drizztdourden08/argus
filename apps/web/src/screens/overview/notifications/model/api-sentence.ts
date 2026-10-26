/* @layer renderer-app @kind logic */
import { ApiError } from '../../../../data/api';
import { DOWN_SENTENCE } from '../../overview.constants';

const apiSentence = (error: unknown, otherwise: (detail: string) => string): string => {
  if (error instanceof ApiError && error.status === 0) return DOWN_SENTENCE;
  if (error instanceof ApiError && error.status === 403) return 'Only this page may change the settings.';
  const detail = error instanceof Error ? error.message : 'unknown';
  return otherwise(detail || 'unknown');
};

export { apiSentence };
