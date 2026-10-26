/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { writeFailure } from '../../../../data/kanban';

const announceFailure = (error: unknown): void => {
  toast({ message: writeFailure(error), variant: 'danger' });
};

export { announceFailure };
