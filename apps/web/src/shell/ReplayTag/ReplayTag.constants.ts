/* @layer renderer-app @kind constants */
import type { ReplayState } from '../../data/remote';

const REPLAY_LOOK: Readonly<Record<ReplayState, { label: string; title: string; color: 'success' | 'warning' | 'danger' }>> = {
  pending: { label: 'waits for the go', title: 'Not on GitHub yet: the replay runs on the owner\'s go', color: 'warning' },
  done: { label: 'on GitHub', title: 'Replayed on GitHub', color: 'success' },
  failed: { label: 'replay failed', title: 'The replay stopped at this record', color: 'danger' },
};

export { REPLAY_LOOK };
