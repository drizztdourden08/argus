/* @layer renderer-app @kind component */
import { Inline, Status, Text } from '@drizztdourden08/tessera';
import type { SyncFooterProps } from '../cards.type';
import { clockText, driftText, lastSyncText } from '../model';

const SyncFooter = ({ drift, run, active }: SyncFooterProps) => {
  const total = drift?.total ?? 0;
  const scanned = clockText(drift?.scannedAt);
  return (
    <>
      <Status tone={total === 0 ? 'success' : 'warning'} size="sm" dot>{driftText(drift)}</Status>
      <Inline gap="md" align="center" wrap>
        {scanned && <Text variant="caption" tone="muted">{`Scanned ${scanned}`}</Text>}
        {!active && <Text variant="caption" tone={run?.phase === 'failed' ? 'danger' : 'muted'}>{lastSyncText(run, total > 0)}</Text>}
      </Inline>
    </>
  );
};

export { SyncFooter };
