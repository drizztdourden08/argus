/* @layer renderer-app @kind logic */
import type { ToastInput } from '@drizztdourden08/tessera';
import { ApiError } from '../../../data/api';
import { DOWN_MESSAGE } from './owner-posts.constants';

const failureToast = (error: unknown): ToastInput => {
  if (error instanceof ApiError && error.code === 'unavailable') return { variant: 'danger', message: DOWN_MESSAGE };
  const reason = error instanceof ApiError ? error.message : 'unknown';
  return { variant: 'danger', message: `Write refused: ${reason}` };
};

export { failureToast };
