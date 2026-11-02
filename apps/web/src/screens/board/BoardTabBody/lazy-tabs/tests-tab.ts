/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadTestsTab = () => import('../../tabs/tests').then((loaded) => ({ default: loaded.TestsTab }));

const LazyTestsTab = lazy(loadTestsTab);

export { LazyTestsTab, loadTestsTab };
