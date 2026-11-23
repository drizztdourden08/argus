/* @layer renderer-app @kind logic */
import type { SignalId } from './board.type';

const signalText = (id: SignalId, count: number): string => {
  switch (id) {
    case 'needs-you': return count === 1 ? '1 needs you' : `${count} need you`;
    case 'failing': return count === 1 ? '1 failing test' : `${count} failing tests`;
    case 'blocked': return `${count} blocked`;
    case 'stalled': return `${count} stalled`;
    case 'sync-due': return 'Sync due';
    case 'master-missing': return 'Master missing';
    case 'paused': return `${count} paused`;
    case 'archived': return 'Archived';
  }
};

export { signalText };
