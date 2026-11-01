/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyReleasesTab = lazy(() => import('../releases/ReleasesTab').then((loaded) => ({ default: loaded.ReleasesTab })));

export { LazyReleasesTab };
