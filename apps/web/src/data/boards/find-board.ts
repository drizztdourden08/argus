/* @layer renderer-app @kind logic */
import type { BoardInfo } from './boards.type';

const findBoard = (boards: readonly BoardInfo[], idOrAlias: string): BoardInfo | undefined =>
  boards.find((board) => board.id === idOrAlias) ?? boards.find((board) => board.aliases.includes(idOrAlias));

export { findBoard };
