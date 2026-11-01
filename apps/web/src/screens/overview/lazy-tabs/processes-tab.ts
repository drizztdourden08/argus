/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyProcessesTab = lazy(() => import('../processes/ProcessesTab').then((loaded) => ({ default: loaded.ProcessesTab })));

export { LazyProcessesTab };
