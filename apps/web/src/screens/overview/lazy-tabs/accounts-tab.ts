/* @layer renderer-app @kind component */
import { lazy } from 'react';
import { OVERVIEW_TAB_LOADERS } from './loaders.constants';

const LazyAccountsTab = lazy(OVERVIEW_TAB_LOADERS.accounts);

export { LazyAccountsTab };
