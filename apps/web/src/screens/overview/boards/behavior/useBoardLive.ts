/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { PORTFOLIO_SCOPE, useCollection } from '../../../../data/react';
import { useRemoteSyncs } from '../../../../data/remote';
import { stallsOf, useStalls } from '../../../../data/stalls';
import type { RegistryRow } from '../../../../data/types';
import { syncTone } from '../../../../shell/SyncStamp/behavior/sync-tone';
import type { BoardLive } from '../model';
import { faultOf } from './fault-of';

const useBoardLive = (now: number): ((key: string) => BoardLive) => {
  const registry = useCollection<RegistryRow>(PORTFOLIO_SCOPE, 'registry').docs;
  const stalls = useStalls().docs;
  const syncs = useRemoteSyncs();
  return useMemo(
    () => (key) => {
      const sync = syncs?.[key];
      return { fault: faultOf(registry, key), stalls: stallsOf(stalls, key).length, syncDue: sync ? syncTone(sync.at, now) === 'danger' : false };
    },
    [registry, stalls, syncs, now],
  );
};

export { useBoardLive };
