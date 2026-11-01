/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyCacheTab = lazy(() => import('../cache/CacheTab').then((loaded) => ({ default: loaded.CacheTab })));

export { LazyCacheTab };
