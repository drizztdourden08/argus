/* @layer renderer-app @kind types */
import type { BoardSignal } from './board.type';

type Raw = Omit<BoardSignal, 'text'>;

export type { Raw };
