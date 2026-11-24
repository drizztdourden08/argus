/* @layer renderer-app @kind component */
import { Card, Stack, Text } from '@drizztdourden08/tessera';
import type { SyncCardProps } from '../cards.type';
import { SYNC_TITLE } from '../compare.constants';
import { isRunActive, syncSentence } from '../model';
import { SyncActions } from './SyncActions';
import { SyncChoices } from './SyncChoices';
import { SyncFooter } from './SyncFooter';
import { SyncRunLine } from './SyncRunLine';

const SyncCard = (props: SyncCardProps) => {
  const { drift, accounts, run, steps, actions, onReview } = props;
  const active = isRunActive(run);
  return (
    <Card title={SYNC_TITLE} actions={<SyncActions run={run} actions={actions} onReview={onReview} />}>
      <Stack gap="xs">
        <Text>{syncSentence(accounts)}</Text>
        <SyncChoices accounts={accounts} disabled={actions.busy || active} actions={actions} onReview={onReview} />
        {run && active && <SyncRunLine run={run} steps={steps} accounts={accounts} />}
        <SyncFooter drift={drift} run={run} active={active} />
      </Stack>
    </Card>
  );
};

export { SyncCard };
