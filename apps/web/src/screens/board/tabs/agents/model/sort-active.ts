/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';
import { startMs } from './start-ms';

const byStart = (a: WithId<Agent>, b: WithId<Agent>): number =>
  (startMs(a) || 0) - (startMs(b) || 0) || a.id.localeCompare(b.id, undefined, { numeric: true });

const sortActive = (agents: readonly WithId<Agent>[]): WithId<Agent>[] => [...agents].sort(byStart);

export { sortActive };
