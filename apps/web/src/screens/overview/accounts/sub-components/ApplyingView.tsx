/* @layer renderer-app @kind component */
import { Stack } from '@drizztdourden08/tessera';
import { useCompareFeed } from '../behavior/useCompareFeed';
import { useRunFeed } from '../behavior/useRunFeed';
import type { ApplyingViewProps } from '../compare.type';
import { ApplyingProgress } from './ApplyingProgress';
import { ApplyingTable } from './ApplyingTable';
import { RunFinish } from './RunFinish';
import { StepKindTags } from './StepKindTags';
import { StepLog } from './StepLog';

const ApplyingView = (props: ApplyingViewProps) => {
  const { actions, onBack } = props;
  const { run, steps } = useRunFeed();
  const feed = useCompareFeed();
  return (
    <Stack gap="md">
      <ApplyingProgress run={run} />
      <StepKindTags steps={steps} />
      <RunFinish phase={run?.phase ?? 'idle'} result={run?.result} busy={actions.busy} onUndo={actions.undo} onBack={onBack} />
      <ApplyingTable data={feed.data} steps={steps} />
      <StepLog steps={steps} accounts={feed.data?.accounts ?? []} />
    </Stack>
  );
};

export { ApplyingView };
