/* @layer renderer-app @kind logic */
import { boardsStore } from './boards-store';
import { findBoard } from './find-board';

const boardName = (idOrAlias: string): string => findBoard(boardsStore.get(), idOrAlias)?.name ?? idOrAlias;

export { boardName };
