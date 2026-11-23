/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { ProjectThread } from '../../../../data/agents';
import { useCollection, useDoc } from '../../../../data/react';
import type { RegistryRow } from '../../../../data/types';
import type { WorktreesMeta } from '../../../../data/worktrees';
import { useAgeClock } from '../../../../shell/Age/behavior/useAgeClock';
import { findLead } from '../../tabs/agents/model';
import type { BoardHealthView, BoardHeaderProps } from '../BoardHeader.type';
import { healthOf } from './board-health';

const useBoardHealth = (props: Pick<BoardHeaderProps, 'boardKey' | 'status' | 'updatedAt'>): BoardHealthView => {
  const { boardKey, status, updatedAt } = props;
  const threads = useCollection<ProjectThread>(boardKey, 'threads');
  const registry = useCollection<RegistryRow>(boardKey, 'registry');
  const counts = useDoc<WorktreesMeta>(boardKey, 'meta', 'worktrees').doc;
  const now = useAgeClock();
  const lead = findLead(threads.docs);
  const faults = useMemo(() => registry.docs.filter((row) => row.state === 'fault'), [registry.docs]);
  const offline = status === 'unavailable' || status === 'unauthorized';
  const health = healthOf({ lead, faults: faults.length, offline, updatedAt, now });
  return { ...health, lead, faults, listed: counts?.listed ?? undefined, runningAgents: counts?.runningAgents ?? undefined, status, updatedAt };
};

export { useBoardHealth };
