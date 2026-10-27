/* @layer renderer-app @kind component */
import { Stack } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { RunLogDrawer } from '../../board/run-terminal';
import { useRunningRuns } from './behavior/useRunningRuns';
import { useSendDown } from './behavior/useSendDown';
import type { DashboardsTabProps, LogTarget } from './dashboards.type';
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
  const [log, setLog] = useState<LogTarget | null>(null);
  return (
    <Stack gap="md">
      <DashboardTotals cards={cards} status={status} onSend={send.open} />
      <RegistryStrip />
      <RunningNow runs={runs} />
      <ProjectGrid cards={cards} onSend={send.open} onLog={(key, run) => setLog({ key, run })} />
      <SentDown />
      <SendDownDialog send={send} />
      <RunLogDrawer run={log?.run ?? null} boardKey={log?.key ?? ''} onClose={() => setLog(null)} />
    </Stack>
  );
};

export { DashboardsTab };
