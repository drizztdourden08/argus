/* @layer renderer-app @kind logic */
import { useEffect, useRef } from 'react';

const useResetOnBoard = (boardKey: string, reset: () => void): void => {
  const previous = useRef(boardKey);
  const latest = useRef(reset);
  latest.current = reset;
  useEffect(() => {
    if (previous.current === boardKey) return;
    previous.current = boardKey;
    latest.current();
  }, [boardKey]);
};

export { useResetOnBoard };
