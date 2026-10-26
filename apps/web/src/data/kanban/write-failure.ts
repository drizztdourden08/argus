/* @layer renderer-app @kind logic */
import { ApiError } from '../api';
import { UNREACHABLE_MESSAGE } from './kanban.constants';

const writeFailure = (error: unknown): string => {
  if (error instanceof ApiError) return error.code === 'unavailable' ? UNREACHABLE_MESSAGE : `Write refused: ${error.code}`;
  return `Write refused: ${error instanceof Error ? error.message : 'unknown'}`;
};

export { writeFailure };
