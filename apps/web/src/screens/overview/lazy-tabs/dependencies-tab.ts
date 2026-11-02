/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadDependenciesTab = () => import('../deps/DepsTab').then((loaded) => ({ default: loaded.DepsTab }));

const LazyDependenciesTab = lazy(loadDependenciesTab);

export { LazyDependenciesTab, loadDependenciesTab };
