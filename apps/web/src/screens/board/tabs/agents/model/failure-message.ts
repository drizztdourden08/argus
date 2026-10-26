/* @layer renderer-app @kind logic */
import { ApiError } from '../../../../../data/api';
import { UNREACHABLE_TEXT } from './model.constants';

const failureMessage = (error: unknown): string => {
  if (error instanceof ApiError && error.code !== 'unavailable') return `Write refused: ${error.code}`;
  return error instanceof ApiError ? UNREACHABLE_TEXT : 'Write refused: unknown';
};

export { failureMessage };
