/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyRegistryTab = lazy(() => import('../registry/RegistryTab').then((loaded) => ({ default: loaded.RegistryTab })));

export { LazyRegistryTab };
