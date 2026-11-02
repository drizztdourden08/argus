/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadRegistryTab = () => import('../registry/RegistryTab').then((loaded) => ({ default: loaded.RegistryTab }));

const LazyRegistryTab = lazy(loadRegistryTab);

export { LazyRegistryTab, loadRegistryTab };
