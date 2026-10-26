/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import { readStored, writeStored } from '../../../shared';

const useStoredFlag = (key: string): [boolean, (next: boolean) => void] => {
  const [flag, setFlag] = useState(() => readStored(key) === '1');
  const update = useCallback(
    (next: boolean) => {
      writeStored(key, next ? '1' : '0');
      setFlag(next);
    },
    [key],
  );
  return [flag, update];
};

export { useStoredFlag };
