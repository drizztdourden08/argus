/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadCacheTab = () => import('../cache/CacheTab').then((loaded) => ({ default: loaded.CacheTab }));

const LazyCacheTab = lazy(loadCacheTab);

export { LazyCacheTab, loadCacheTab };
