/* @layer renderer-app @kind logic */
import type { ToastInput } from '@drizztdourden08/tessera';
import type { PostResult } from '../../../data/api';
import { CONFLICT_DEFAULT, REFUSED_DEFAULT } from './owner-posts.constants';

const verdictToast = (result: PostResult, said: string): ToastInput => {
  if (result.status === 'conflict') {
    return { variant: 'danger', message: `Conflict: ${result.outcome ?? CONFLICT_DEFAULT}. Nothing of yours was applied.` };
  }
  if (result.status === 'refused') return { variant: 'danger', message: `Refused: ${result.outcome ?? REFUSED_DEFAULT}.` };
  return { variant: 'success', message: said };
};

export { verdictToast };
