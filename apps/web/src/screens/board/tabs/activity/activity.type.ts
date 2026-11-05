/* @layer renderer-app @kind types */
import type { BoardWork } from '../../feed-model';

interface ActivityTabProps {
  boardKey: string;
  work: BoardWork;
}

export type { ActivityTabProps };
