/* @layer renderer-app @kind logic */
import { ApiError } from '../../../../data/api';
import { DOWN_SENTENCE } from '../../overview.constants';

const cacheError = (error: unknown, otherwise: (detail: string) => string): string => {
  if (error instanceof ApiError && error.status === 0) return DOWN_SENTENCE;
  if (error instanceof ApiError && error.code === 'read_only') return 'Argus runs in shadow mode here and deletes nothing.';
  if (error instanceof ApiError && error.status === 409) return `Nothing was deleted: ${error.message}.`;
  const detail = error instanceof Error ? error.message : 'unknown';
  return otherwise(detail || 'unknown');
};

export { cacheError };
