/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyReleasesTab = lazy(() => import('../../tabs/releases').then((loaded) => ({ default: loaded.ReleasesTab })));

export { LazyReleasesTab };
