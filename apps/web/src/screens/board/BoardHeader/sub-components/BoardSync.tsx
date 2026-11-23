/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { useRemoteSyncs } from '../../../../data/remote';
import { Age } from '../../../../shell';
import type { BoardSyncProps } from '../BoardHeader.type';

const BoardSync = (props: BoardSyncProps) => {
  const sync = useRemoteSyncs()?.[props.boardKey] ?? null;
  if (!sync) return <Text tone="muted">never</Text>;
  return <Text title={sync.note ?? undefined}><Age value={sync.at} ago /></Text>;
};

export { BoardSync };
