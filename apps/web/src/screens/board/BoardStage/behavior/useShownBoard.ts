/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { prefersReducedMotion } from './prefers-reduced-motion';
import { runBoardReveal } from './run-board-reveal';

const useShownBoard = (boardKey: string, ready: boolean): string => {
  const [shown, setShown] = useState(boardKey);
  useEffect(() => {
    if (shown === boardKey) return undefined;
    return runBoardReveal({ ready, reduced: prefersReducedMotion(), reveal: () => setShown(boardKey) });
  }, [boardKey, ready, shown]);
  return shown;
};

export { useShownBoard };
