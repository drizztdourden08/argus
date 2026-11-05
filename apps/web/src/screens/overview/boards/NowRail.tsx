/* @layer renderer-app @kind component */
import { Stack } from '@drizztdourden08/tessera';
import { useRunningRuns } from './behavior/useRunningRuns';
import { RunningNow } from './sub-components/RunningNow';
import { SentDown } from './sub-components/SentDown';
import type { EntriesProps } from './boards-view.type';
import { WaitingOnYou } from './sub-components/WaitingOnYou';

const NowRail = (props: EntriesProps) => {
  const { entries } = props;
  const runs = useRunningRuns();
  return (
    <Stack gap="md">
      <RunningNow runs={runs} />
      <WaitingOnYou entries={entries} />
      <SentDown />
    </Stack>
  );
};

export { NowRail };
