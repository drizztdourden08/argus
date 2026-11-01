/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyChangesTab = lazy(() => import('../../tabs/changes').then((loaded) => ({ default: loaded.ChangesTab })));

export { LazyChangesTab };
