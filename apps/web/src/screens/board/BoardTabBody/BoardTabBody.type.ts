/* @layer renderer-app @kind types */
import type { BoardMoreState } from '../BoardMore/BoardMore.type';
import type { BoardWork } from '../feed-model';

interface BoardTabBodyProps {
  tab: string;
  boardKey: string;
  work: BoardWork;
  more: BoardMoreState;
}

export type { BoardTabBodyProps };
