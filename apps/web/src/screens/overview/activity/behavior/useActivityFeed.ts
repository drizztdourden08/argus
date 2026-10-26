/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { PORTFOLIO_SCOPE, useCollection } from '../../../../data/react';
import type { ActivityDoc } from '../../../../data/types';
import { sortActivity } from '../model';
import type { ActivityItem } from '../model';

const useActivityFeed = (): ActivityItem[] => {
  const { docs } = useCollection<ActivityDoc>(PORTFOLIO_SCOPE, 'activity');
  return useMemo(() => sortActivity(docs), [docs]);
};

export { useActivityFeed };
