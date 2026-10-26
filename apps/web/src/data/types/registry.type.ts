/* @layer renderer-app @kind types */
type RegistryState = 'fault' | 'waking' | 'busy' | 'listening' | 'idle';

interface RegistryFault {
  since: string;
  reason: string;
  wakeId?: string;
  detail?: string;
}

interface RegistryPending {
  id?: string;
  since: string;
  reason?: string;
}

interface RegistryRow {
  id: string;
  project: string;
  thread: string;
  session?: string | null;
  role?: string;
  listening?: boolean;
  listenerLabel?: string | null;
  since?: string | null;
  state: RegistryState;
  pendingWake?: RegistryPending | null;
  fault?: RegistryFault | null;
  lastSeen?: string | null;
  activity?: string | null;
  deaf?: boolean;
  boardRequired?: boolean;
  boardOpen?: boolean;
}

interface SentEntry {
  kind?: string;
  author?: string;
  project?: string;
  text?: string;
  createdAt?: string;
  status?: string;
}

export type { RegistryFault, RegistryRow, SentEntry };
