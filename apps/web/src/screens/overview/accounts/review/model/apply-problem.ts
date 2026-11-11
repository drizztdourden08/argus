/* @layer renderer-app @kind logic */
import { ApiError, errorMessage } from '../../../../../data/api';
import { BUSY_TEXT, READ_ONLY_TEXT } from '../review.constants';

const applyProblem = (failure: unknown): string => {
  if (failure instanceof ApiError) {
    if (failure.status === 403 || failure.code === 'read_only') return READ_ONLY_TEXT;
    if (failure.status === 409) return BUSY_TEXT;
  }
  return errorMessage(failure);
};

export { applyProblem };
