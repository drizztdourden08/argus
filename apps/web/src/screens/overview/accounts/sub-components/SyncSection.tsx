/* @layer renderer-app @kind component */
import { Card, Stack } from '@drizztdourden08/tessera';
import type { SyncSectionProps } from '../accounts.type';
import { useRunFeed } from '../behavior/useRunFeed';
import { SYNC_TITLE } from '../compare.constants';
import { isRunActive } from '../model';
import { SyncActions } from './SyncActions';
import { SyncRunLine } from './SyncRunLine';
import { SyncStateLine } from './SyncStateLine';

const SyncSection = (props: SyncSectionProps) => {
  const { drift, accounts, actions, onReview } = props;
  const { run, steps } = useRunFeed();
  const live = isRunActive(run) ? run : null;
  const active = live !== null;
  return (
    <Card title={SYNC_TITLE} actions={<SyncActions run={run} active={active} actions={actions} onReview={onReview} />}>
      <Stack gap="xs">
        <SyncStateLine drift={drift} run={run} active={active} />
        {live && <SyncRunLine run={live} steps={steps} accounts={accounts} />}
      </Stack>
    </Card>
  );
};

export { SyncSection };
