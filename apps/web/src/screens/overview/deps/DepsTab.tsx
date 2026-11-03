/* @layer renderer-app @kind component */
import { EmptyState, Stack } from '@drizztdourden08/tessera';
import { useMemo, useState } from 'react';
import { useDeps } from '../../../data/deps';
import { useNow } from '../clock';
import { NO_DEPS } from './deps.constants';
import { filterDeps, sortDeps } from './model';
import type { DepFilter } from './model';
import { DepsFilter } from './sub-components/DepsFilter';
import { DepsTable } from './sub-components/DepsTable';

const DepsTab = () => {
  const { docs } = useDeps();
  const now = useNow();
  const [filter, setFilter] = useState<DepFilter>('open');
  const rows = useMemo(() => sortDeps(filterDeps(docs, filter), now), [docs, filter, now]);
  return (
    <Stack gap="md">
      <DepsFilter value={filter} onChange={setFilter} />
      {rows.length === 0 ? <EmptyState message={NO_DEPS[filter]} /> : <DepsTable docs={rows} now={now} />}
    </Stack>
  );
};

export { DepsTab };
