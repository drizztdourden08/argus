/* @layer renderer-app @kind types */
import type { DepHistoryRow, DepState } from '../../../../../data/deps';

interface DepLink {
  side: 'waits' | 'asked';
  dep: string;
  project: string;
  item: string;
  title: string;
  thread: string | null;
  state: DepState;
  since: string | null;
  reason: string | null;
}

type DepView = DepLink & {
  history: DepHistoryRow[];
  cancellable: boolean;
};

export type { DepLink, DepView };
