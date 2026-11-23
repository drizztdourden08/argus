/* @layer renderer-app @kind component */
import { Inline, Status, Text } from '@drizztdourden08/tessera';
import { DRIFT_TIP } from '../accounts.constants';
import type { SyncStateLineProps } from '../accounts.type';
import { MATCHING } from '../compare.constants';
import { clockText, lastSyncText } from '../model';

const SyncStateLine = (props: SyncStateLineProps) => {
  const { drift, run, active } = props;
  const total = drift?.total ?? 0;
  const scanned = clockText(drift?.scannedAt);
  return (
    <Inline gap="md" align="center" wrap>
      <Status tone={total === 0 ? 'success' : 'warning'} size="sm" dot title={DRIFT_TIP}>{total === 0 ? MATCHING : `${total} to sync`}</Status>
      {scanned && <Text variant="caption" tone="muted">{`Scanned ${scanned}`}</Text>}
      {!active && <Text variant="caption" tone={run?.phase === 'failed' ? 'danger' : 'muted'}>{lastSyncText(run)}</Text>}
    </Inline>
  );
};

export { SyncStateLine };
