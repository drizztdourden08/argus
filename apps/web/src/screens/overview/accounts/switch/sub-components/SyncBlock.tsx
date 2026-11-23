/* @layer renderer-app @kind component */
import { Inline, Stack, Status, Text } from '@drizztdourden08/tessera';
import { countsText } from '../model';
import { SYNC_STATUSES, SYNC_TITLE, SYNC_UNSTARTED } from '../switch.constants';
import type { SyncBlockProps } from '../switch.type';

const SyncBlock = (props: SyncBlockProps) => {
  const { sync } = props;
  const counts = countsText(sync?.counts);
  return (
    <Stack gap="2xs" as="section" aria-label={SYNC_TITLE}>
      <Text variant="overline" tone="muted">{SYNC_TITLE}</Text>
      {sync?.state ? (
        <Inline gap="sm" align="center" wrap>
          <Status map={SYNC_STATUSES} value={sync.state} size="sm" dot />
          {counts && <Text variant="caption" tone="muted">{counts}</Text>}
        </Inline>
      ) : <Text variant="caption" tone="faint">{SYNC_UNSTARTED}</Text>}
      {sync?.backupDir && <Text variant="caption" tone="muted" mono clamp={1} title={sync.backupDir}>{`Backup: ${sync.backupDir}`}</Text>}
      {sync?.error && <Text variant="caption" tone="danger">{sync.error}</Text>}
    </Stack>
  );
};

export { SyncBlock };
