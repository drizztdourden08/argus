/* @layer renderer-app @kind component */
import { Stack } from '@drizztdourden08/tessera';
import { useRunningRuns } from './behavior/useRunningRuns';
import { useSendDown } from './behavior/useSendDown';
import type { DashboardsTabProps } from './dashboards.type';
import { DashboardTotals } from './sub-components/DashboardTotals';
import { ProjectGrid } from './sub-components/ProjectGrid';
import { RegistryStrip } from './sub-components/RegistryStrip';
import { RunningNow } from './sub-components/RunningNow';
import { SendDownDialog } from './sub-components/SendDownDialog';
import { SentDown } from './sub-components/SentDown';

const DashboardsTab = (props: DashboardsTabProps) => {
  const { cards, status } = props;
  const runs = useRunningRuns();
  const send = useSendDown();
  return (
    <Stack gap="md">
      <DashboardTotals cards={cards} status={status} onSend={send.open} />
      <RegistryStrip />
      <RunningNow runs={runs} />
      <ProjectGrid cards={cards} onSend={send.open} />
      <SentDown />
      <SendDownDialog send={send} />
    </Stack>
  );
};

export { DashboardsTab };
