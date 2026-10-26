/* @layer renderer-app @kind logic */
import { ApiError } from '../../../../../data/api';
import { UNREACHABLE } from './model.constants';

const failureText = (error: unknown, prefix: string): string => {
  if (error instanceof ApiError && error.code === 'unavailable') return UNREACHABLE;
  const message = error instanceof Error && error.message ? error.message : 'refused';
  return `${prefix}: ${message}`;
};

export { failureText };
