/* @layer renderer-app @kind component */
import { lazy } from 'react';

const LazyAgentsTab = lazy(() => import('../../tabs/agents').then((loaded) => ({ default: loaded.AgentsTab })));

export { LazyAgentsTab };
