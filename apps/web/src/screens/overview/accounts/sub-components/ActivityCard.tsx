/* @layer renderer-app @kind component */
import { Card, Stack, Text } from '@drizztdourden08/tessera';
import { ACTIVITY_TITLE } from '../accounts.constants';
import type { ActivityCardProps } from '../cards.type';
import { currentName, isRunActive, lastSyncText } from '../model';
import { ProcessPanel, SwitchConfirm } from '../switch';
import { SyncRunLine } from './SyncRunLine';

const ActivityCard = (props: ActivityCardProps) => {
  const { doc, accounts, confirmKey, drifted, run, steps, onCloseConfirm } = props;
  const target = confirmKey ? accounts.find((account) => account.id === confirmKey || account.key === confirmKey) : undefined;
  const live = run && isRunActive(run) ? run : null;
  return (
    <Card title={ACTIVITY_TITLE}>
      <Stack gap="md">
        {target && <SwitchConfirm target={target} from={currentName(accounts)} onClose={onCloseConfirm} />}
        <ProcessPanel doc={doc} accounts={accounts} />
        {live
          ? <SyncRunLine run={live} steps={steps} accounts={accounts} />
          : <Text variant="caption" tone={run?.phase === 'failed' ? 'danger' : 'muted'}>{lastSyncText(run, drifted)}</Text>}
      </Stack>
    </Card>
  );
};

export { ActivityCard };
