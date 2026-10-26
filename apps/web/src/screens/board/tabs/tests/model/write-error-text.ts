/* @layer renderer-app @kind logic */
import { ApiError } from '../../../../../data/api';

const UNREACHABLE_TEXT = 'Dashboard unreachable, retrying.';

const writeErrorText = (error: unknown): string => {
  if (error instanceof ApiError) return error.code === 'unavailable' ? UNREACHABLE_TEXT : `Write refused: ${error.code}`;
  return `Write refused: ${error instanceof Error ? error.message : 'unknown'}`;
};

export { writeErrorText };
