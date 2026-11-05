/* @layer renderer-app @kind logic */
import type { BoardCounts } from '../model';

const agentsText = (counts: BoardCounts): string => {
  if (counts.agents === 0) return 'no agents';
  const noun = counts.agents === 1 ? 'agent' : 'agents';
  return `${counts.agents} ${noun} · ${counts.working} working`;
};

export { agentsText };
