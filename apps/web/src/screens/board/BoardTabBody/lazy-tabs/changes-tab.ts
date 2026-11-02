/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadChangesTab = () => import('../../tabs/changes').then((loaded) => ({ default: loaded.ChangesTab }));

const LazyChangesTab = lazy(loadChangesTab);

export { LazyChangesTab, loadChangesTab };
