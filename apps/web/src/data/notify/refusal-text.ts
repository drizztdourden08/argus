/* @layer renderer-app @kind logic */
import { ApiError, errorMessage } from '../api';
import { FORBIDDEN_LINE } from './settings-store.constants';

const refusalText = (failure: unknown): string =>
  failure instanceof ApiError && (failure.status === 403 || failure.code === 'forbidden') ? FORBIDDEN_LINE : errorMessage(failure);

export { refusalText };
