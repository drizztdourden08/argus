/* @layer renderer-app @kind types */
import type { Thread } from '../../data/types';

interface BoardPageProps {
  boardKey: string;
}

interface ThreadListProps {
  threads: Thread[];
}

export type { BoardPageProps, ThreadListProps };
