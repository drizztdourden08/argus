/* @layer renderer-app @kind component */
import { Card, Inline, Stack, Status, Text } from '@drizztdourden08/tessera';
import type { SyncCardProps } from '../cards.type';
import { SYNC_TITLE } from '../compare.constants';
import { clockText, driftText, isRunActive, lastSyncText, syncSentence } from '../model';
import { SyncActions } from './SyncActions';
import { SyncChoices } from './SyncChoices';

const SyncCard = (props: SyncCardProps) => {
  const { drift, accounts, run, actions, onReview } = props;
  const total = drift?.total ?? 0;
  const scanned = clockText(drift?.scannedAt);
  const active = isRunActive(run);
  return (
    <Card title={SYNC_TITLE} actions={<SyncActions run={run} actions={actions} onReview={onReview} />}>
      <Stack gap="xs">
        <Text>{syncSentence(accounts)}</Text>
        <SyncChoices accounts={accounts} disabled={actions.busy || active} actions={actions} />
        <Status tone={total === 0 ? 'success' : 'warning'} size="sm" dot>{driftText(drift)}</Status>
        <Inline gap="md" align="center" wrap>
          {scanned && <Text variant="caption" tone="muted">{`Scanned ${scanned}`}</Text>}
          {!active && <Text variant="caption" tone={run?.phase === 'failed' ? 'danger' : 'muted'}>{lastSyncText(run, total > 0)}</Text>}
        </Inline>
      </Stack>
    </Card>
  );
};

export { SyncCard };
