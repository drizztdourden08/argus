/* @layer renderer-app @kind logic */
import { useCallback, useEffect, useState } from 'react';
import { readStored } from './read-stored';
import { watchStored } from './watch-stored';
import { writeStored } from './write-stored';

const useStored = (key: string): [string, (next: string) => void] => {
  const [value, setValue] = useState(() => readStored(key) ?? '');
  useEffect(() => watchStored(() => setValue(readStored(key) ?? '')), [key]);
  const update = useCallback(
    (next: string) => {
      setValue(next);
      writeStored(key, next);
    },
    [key],
  );
  return [value, update];
};

export { useStored };
