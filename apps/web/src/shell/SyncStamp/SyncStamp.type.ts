/* @layer renderer-app @kind types */
import type { RemoteSync } from '../../data/remote';

type SyncTone = 'muted' | 'warning' | 'danger';

interface SyncStampProps {
  sync: RemoteSync | null;
  compact?: boolean;
}

export type { SyncStampProps, SyncTone };
