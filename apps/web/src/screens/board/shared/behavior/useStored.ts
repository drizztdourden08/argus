/* @layer renderer-app @kind logic */
import { useCallback, useEffect, useState } from 'react';
import { readStored } from './read-stored';
import { watchStored } from './watch-stored';
import { writeStored } from './write-stored';

const useStored = (key: string): [string, (next: string) => void] => {
  const [held, setHeld] = useState(() => ({ key, value: readStored(key) ?? '' }));
  useEffect(() => watchStored(() => setHeld({ key, value: readStored(key) ?? '' })), [key]);
  const update = useCallback(
    (next: string) => {
      setHeld({ key, value: next });
      writeStored(key, next);
    },
    [key],
  );
  return [held.key === key ? held.value : (readStored(key) ?? ''), update];
};

export { useStored };
