/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { useStored } from './useStored';

const useStoredFlag = (key: string): [boolean, (next: boolean) => void] => {
  const [value, set] = useStored(key);
  const update = useCallback((next: boolean) => set(next ? '1' : '0'), [set]);
  return [value === '1', update];
};

export { useStoredFlag };
