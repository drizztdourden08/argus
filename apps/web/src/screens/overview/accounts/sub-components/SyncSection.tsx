/* @layer renderer-app @kind component */
import { Button, Card, Inline, Stack, Status, Text } from '@drizztdourden08/tessera';
import { DRIFT_TIP } from '../accounts.constants';
import type { SyncSectionProps } from '../accounts.type';
import { useRunFeed } from '../behavior/useRunFeed';
import { ACTIVE_PHASES, MATCHING, REVIEW_LABEL, SYNC_LABEL, SYNC_TITLE, UNDO_LABEL } from '../compare.constants';
import { clockText, lastSyncText } from '../model';
import { SyncRunLine } from './SyncRunLine';

const SyncSection = (props: SyncSectionProps) => {
  const { drift, accounts, actions, onReview } = props;
  const { run, steps } = useRunFeed();
  const total = drift?.total ?? 0;
  const active = run?.phase !== undefined && ACTIVE_PHASES.has(run.phase);
  const finished = run?.phase === 'done' || run?.phase === 'failed';
  const scanned = clockText(drift?.scannedAt);
  const buttons = (
    <Inline gap="xs">
      <Button size="xs" variant="ghost" onClick={onReview}>{REVIEW_LABEL}</Button>
      {finished && <Button size="xs" variant="ghost" disabled={actions.busy} onClick={actions.undo}>{UNDO_LABEL}</Button>}
      <Button size="xs" disabled={actions.busy || active} onClick={actions.sync}>{SYNC_LABEL}</Button>
    </Inline>
  );
  return (
    <Card title={SYNC_TITLE} actions={buttons}>
      <Stack gap="xs">
        <Inline gap="md" align="center" wrap>
          <Status tone={total === 0 ? 'success' : 'warning'} size="sm" dot title={DRIFT_TIP}>{total === 0 ? MATCHING : `${total} to sync`}</Status>
          {scanned && <Text variant="caption" tone="muted">{`Scanned ${scanned}`}</Text>}
          {!active && <Text variant="caption" tone={run?.phase === 'failed' ? 'danger' : 'muted'}>{lastSyncText(run)}</Text>}
        </Inline>
        {active && run && <SyncRunLine run={run} steps={steps} accounts={accounts} />}
      </Stack>
    </Card>
  );
};

export { SyncSection };
