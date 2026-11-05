/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import type { KeyboardEvent } from 'react';
import { textProblem } from '../../../owner-posts';

const overLimitKeys = (text: string) => (event: KeyboardEvent<HTMLElement>): void => {
  if (event.key !== 'Enter' || !(event.ctrlKey || event.metaKey)) return;
  const over = textProblem(text);
  if (over) toast({ variant: 'warning', message: over });
};

export { overLimitKeys };
