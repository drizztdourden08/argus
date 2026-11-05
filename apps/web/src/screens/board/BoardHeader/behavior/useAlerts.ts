/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection } from '../../../../data/react';
import type { StreamStatus } from '../../../../data/stream';
import { stallsOf, useStalls } from '../../../../data/stalls';
import type { RegistryRow, Wake } from '../../../../data/types';
import type { AlertsView } from '../BoardHeader.type';
import { passiveNotices } from './passive-notices';

const useAlerts = (boardKey: string, status: StreamStatus): AlertsView => {
  const wakes = useCollection<Wake>(boardKey, 'wakes');
  const registry = useCollection<RegistryRow>(boardKey, 'registry');
  const stallDocs = useStalls().docs;
  const notices = useMemo(() => passiveNotices(wakes.docs), [wakes.docs]);
  const stalls = useMemo(() => stallsOf(stallDocs, boardKey), [stallDocs, boardKey]);
  const faults = useMemo(() => registry.docs.filter((row) => row.state === 'fault'), [registry.docs]);
  const offline = status === 'unavailable' || status === 'unauthorized' ? status : null;
  const count = stalls.length + faults.length + notices.length + (offline ? 1 : 0);
  const tone = offline || stalls.length > 0 || faults.length > 0 ? 'danger' : 'info';
  return { stalls, faults, notices, offline, count, tone };
};

export { useAlerts };
