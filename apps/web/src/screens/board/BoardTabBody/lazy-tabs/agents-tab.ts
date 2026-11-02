/* @layer renderer-app @kind component */
import { lazy } from 'react';

const loadAgentsTab = () => import('../../tabs/agents').then((loaded) => ({ default: loaded.AgentsTab }));

const LazyAgentsTab = lazy(loadAgentsTab);

export { LazyAgentsTab, loadAgentsTab };
