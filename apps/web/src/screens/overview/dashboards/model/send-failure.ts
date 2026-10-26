/* @layer renderer-app @kind logic */
import { ApiError } from '../../../../data/api';
import { DOWN_SENTENCE } from '../../overview.constants';

const sendFailure = (failure: unknown): string => {
  if (failure instanceof ApiError) return failure.status === 0 ? DOWN_SENTENCE : `Write refused: ${failure.code}`;
  return `Write refused: ${failure instanceof Error ? failure.message : 'unknown'}`;
};

export { sendFailure };
