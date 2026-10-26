/* @layer renderer-app @kind types */
type RegistryState = 'fault' | 'waking' | 'busy' | 'listening' | 'idle';

interface RegistryFault {
  since: string;
  reason: string;
}

interface RegistryRow {
  id: string;
  project: string;
  thread: string;
  session?: string;
  role?: string;
  state: RegistryState;
  since?: string | null;
  listenerLabel?: string | null;
  pendingWake?: { since: string } | null;
  fault?: RegistryFault | null;
  deaf?: boolean;
  boardRequired?: boolean;
  boardOpen?: boolean;
}

export type { RegistryFault, RegistryRow };
