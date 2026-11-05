/* @layer renderer-app @kind types */
import type { Card } from '../../../data/types';
import type { BoardOption } from '../BoardSwitcher.type';

interface BoardList {
  cards: Card[];
  options: BoardOption[];
}

export type { BoardList };
