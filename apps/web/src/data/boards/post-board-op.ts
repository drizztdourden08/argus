/* @layer renderer-app @kind logic */
import type { PostResult } from '../api';
import { ownerEntry, postEntry } from '../posts';
import { PORTFOLIO_SCOPE } from '../react';
import { boardEntry } from './board-entry';
import type { BoardOp } from './boards.type';

const postBoardOp = (op: BoardOp): Promise<PostResult> => postEntry(PORTFOLIO_SCOPE, ownerEntry(boardEntry(op)));

export { postBoardOp };
