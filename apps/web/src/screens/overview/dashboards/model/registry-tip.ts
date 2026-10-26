/* @layer renderer-app @kind logic */
import { formatAgo } from '../../format';
import type { RegistryFault } from '../../../../data/types';

const registryTip = (fault: RegistryFault, main: boolean, now: number): string => {
  const since = formatAgo(fault.since, now);
  if (fault.reason === 'board-closed') {
    const tail = main ? '.' : ' and it clears itself when the board opens.';
    return `Board closed since ${since} ago: this thread has no open board tab. The Overlord is resolving it with the thread${tail}`;
  }
  const words = String(fault.reason || 'unknown').replace(/-/g, ' ');
  return `Wake failed: ${words} since ${since} ago. Only the Overlord can clear this${main ? '.' : '; it has been notified.'}`;
};

export { registryTip };
