/* @layer renderer-app @kind component */
import { Callout, Stack } from '@drizztdourden08/tessera';
import { useMemo, useState } from 'react';
import { useCompareFeed } from '../behavior/useCompareFeed';
import { usePlanFlow } from '../behavior/usePlanFlow';
import type { CompareViewProps, RowFilter } from '../compare.type';
import { visibleRows } from '../model';
import { CompareBody } from './CompareBody';
import { CompareToolbar } from './CompareToolbar';

const CompareView = (props: CompareViewProps) => {
  const { actions, onBack } = props;
  const feed = useCompareFeed();
  const plan = usePlanFlow();
  const [filter, setFilter] = useState<RowFilter>({ showSame: false, showArchived: false });
  const rows = useMemo(() => visibleRows(feed.data?.rows ?? [], filter), [feed.data, filter]);
  const busy = feed.loading || plan.busy || actions.busy;
  return (
    <Stack gap="md">
      <CompareToolbar filter={filter} busy={busy} onFilter={setFilter} onBack={onBack} onRescan={feed.rescan} onPlan={plan.run} onSync={actions.sync} />
      {feed.error && <Callout tone="danger">{feed.error}</Callout>}
      {plan.note && <Callout tone="info" title="Plan">{plan.note}</Callout>}
      {feed.data && <CompareBody data={feed.data} rows={rows} />}
    </Stack>
  );
};

export { CompareView };
