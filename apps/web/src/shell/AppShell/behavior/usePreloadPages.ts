/* @layer renderer-app @kind logic */
import { useEffect } from 'react';
import { PAGE_LOADERS } from '../../../routing/lazy-pages/loaders.constants';

const usePreloadPages = (): void => {
  useEffect(() => {
    for (const load of Object.values(PAGE_LOADERS)) void load();
  }, []);
};

export { usePreloadPages };
