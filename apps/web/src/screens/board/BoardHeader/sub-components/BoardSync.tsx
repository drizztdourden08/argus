/* @layer renderer-app @kind component */
import { useRemoteSyncs } from '../../../../data/remote';
import { SyncStamp } from '../../../../shell';
import type { BoardSyncProps } from '../BoardHeader.type';

const BoardSync = (props: BoardSyncProps) => {
  const syncs = useRemoteSyncs();
  return syncs ? <SyncStamp compact sync={syncs[props.boardKey] ?? null} /> : null;
};

export { BoardSync };
