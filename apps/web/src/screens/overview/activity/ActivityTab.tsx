/* @layer renderer-app @kind component */
import { EmptyState, Stack } from '@drizztdourden08/tessera';
import { useMemo, useState } from 'react';
import { NO_FILTERS, NOTHING_YET, PAGE } from './activity.constants';
import type { ActivityTabProps } from './activity.type';
import { useActivityFeed } from './behavior/useActivityFeed';
import { countLevels, filterActivity, kindCounts } from './model';
import type { ActivityFilters } from './model';
import { ActivityFeed } from './sub-components/ActivityFeed';
import { ActivityFilterBar } from './sub-components/ActivityFilterBar';

const ActivityTab = (props: ActivityTabProps) => {
  const { freshSince } = props;
  const items = useActivityFeed();
  const [filters, setFilters] = useState<ActivityFilters>(NO_FILTERS);
  const [shown, setShown] = useState(PAGE);
  const matching = useMemo(() => filterActivity(items, filters), [items, filters]);
  const levels = useMemo(() => countLevels(items), [items]);
  const kinds = useMemo(() => kindCounts(items), [items]);
  if (items.length === 0) return <EmptyState message={NOTHING_YET} />;
  const change = (next: ActivityFilters) => {
    setFilters(next);
    setShown(PAGE);
  };
  return (
    <Stack gap="md">
      <ActivityFilterBar filters={filters} levels={levels} kinds={kinds} total={items.length} onChange={change} />
      <ActivityFeed items={matching} shown={shown} freshSince={freshSince} onMore={() => setShown(shown + PAGE)} />
    </Stack>
  );
};

export { ActivityTab };
