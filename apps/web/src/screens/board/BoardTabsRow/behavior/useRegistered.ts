/* @layer renderer-app @kind logic */
import { useEffect, useId } from 'react';
import type { Registry } from '../BoardTabsRow.type';

const useRegistered = <T>(registry: Registry<T>, value: T): void => {
  const key = useId();
  useEffect(() => {
    registry.set(key, value);
  });
  useEffect(() => () => registry.clear(key), [registry, key]);
};

export { useRegistered };
