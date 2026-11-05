/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { readLastBoard } from './read-last-board';
import { writeLastBoard } from './write-last-board';

const useLastBoard = (current: string | null): string | null => {
  const [last, setLast] = useState(readLastBoard);
  useEffect(() => {
    if (!current) return;
    writeLastBoard(current);
    setLast(current);
  }, [current]);
  return last;
};

export { useLastBoard };
