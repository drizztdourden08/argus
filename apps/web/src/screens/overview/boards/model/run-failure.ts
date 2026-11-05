/* @layer renderer-app @kind logic */
import { ApiError } from '../../../../data/api';

const runFailure = (failure: unknown, down: string): string => {
  if (failure instanceof ApiError) return failure.status === 0 ? down : `Not started: ${failure.message || failure.code}.`;
  return `Not started: ${failure instanceof Error ? failure.message : 'unknown'}.`;
};

export { runFailure };
