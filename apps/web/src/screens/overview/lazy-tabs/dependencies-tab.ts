/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyDependenciesTab = lazy(() => import('../deps/DepsTab').then((loaded) => ({ default: loaded.DepsTab })));

export { LazyDependenciesTab };
