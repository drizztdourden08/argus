/* @layer renderer-app @kind component */
import { ProgressBar, Stack, Text } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { useCompareFeed } from '../behavior/useCompareFeed';
import { useRunFeed } from '../behavior/useRunFeed';
import { PHASE_LABEL } from '../compare.constants';
import type { ApplyingViewProps } from '../compare.type';
import { rowMarks, visibleRows } from '../model';
import { CompareTable } from './CompareTable';
import { RunFinish } from './RunFinish';
import { StepKindTags } from './StepKindTags';
import { StepLog } from './StepLog';

const HIDE_QUIET = { showSame: false, showArchived: false };

const ApplyingView = (props: ApplyingViewProps) => {
  const { actions, onBack } = props;
  const { run, steps } = useRunFeed();
  const feed = useCompareFeed();
  const rows = useMemo(() => visibleRows(feed.data?.rows ?? [], HIDE_QUIET), [feed.data]);
  const marks = useMemo(() => rowMarks(feed.data?.rows ?? [], steps), [feed.data, steps]);
  const phase = run?.phase ?? 'idle';
  const total = run?.total ?? 0;
  return (
    <Stack gap="md">
      <Text size="md" weight={600}>{`Syncing: ${PHASE_LABEL[phase]}`}</Text>
      <ProgressBar value={run?.step ?? 0} max={Math.max(total, 1)} indeterminate={total === 0 && phase !== 'done'} tone={phase === 'failed' ? 'danger' : 'primary'} showValue={total > 0} live />
      <StepKindTags steps={steps} />
      <RunFinish phase={phase} result={run?.result} busy={actions.busy} onUndo={actions.undo} onBack={onBack} />
      {feed.data && rows.length > 0 && <CompareTable accounts={feed.data.accounts} rows={rows} marks={marks} />}
      <StepLog steps={steps} accounts={feed.data?.accounts ?? []} />
    </Stack>
  );
};

export { ApplyingView };
