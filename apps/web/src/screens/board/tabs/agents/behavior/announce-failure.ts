/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { failureMessage } from '../model';

const announceFailure = (error: unknown): void => {
  toast({ message: failureMessage(error), variant: 'danger' });
};

export { announceFailure };
