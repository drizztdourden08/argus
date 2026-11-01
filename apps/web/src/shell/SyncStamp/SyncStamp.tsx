/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { Age } from '../Age';
import { useAgeClock } from '../Age/behavior/useAgeClock';
import { syncTitle } from './behavior/sync-title';
import { syncTone } from './behavior/sync-tone';
import type { SyncStampProps } from './SyncStamp.type';

const SyncStamp = (props: SyncStampProps) => {
  const { sync } = props;
  const now = useAgeClock();
  const tone = syncTone(sync?.at, now);
  return (
    <Text variant="caption" tone={tone} data-tone={tone} title={syncTitle(sync)}>
      last remote sync {sync ? <><Age value={sync.at} /> ago</> : 'never'}
    </Text>
  );
};

export { SyncStamp };
