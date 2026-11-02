/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadProcessesTab = () => import('../processes/ProcessesTab').then((loaded) => ({ default: loaded.ProcessesTab }));

const LazyProcessesTab = lazy(loadProcessesTab);

export { LazyProcessesTab, loadProcessesTab };
