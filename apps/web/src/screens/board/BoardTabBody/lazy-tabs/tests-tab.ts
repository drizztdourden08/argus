/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyTestsTab = lazy(() => import('../../tabs/tests').then((loaded) => ({ default: loaded.TestsTab })));

export { LazyTestsTab };
