/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { postBoardOp } from '../../../../../data/boards';
import type { BoardOp } from '../../../../../data/boards';
import { useWriter } from '../../../../../screens/board/shared';

const useBoardWrite = (): ((op: BoardOp, done: string) => Promise<boolean>) => {
  const write = useWriter();
  return useCallback(
    (op: BoardOp, done: string) =>
      write(async () => {
        const result = await postBoardOp(op);
        return { id: result.id, result };
      }, done),
    [write],
  );
};

export { useBoardWrite };
