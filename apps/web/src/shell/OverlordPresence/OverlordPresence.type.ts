/* @layer renderer-app @kind types */
import type { OverlordActivity, OverlordStatus } from '../../data/overlord';

interface PresenceLook {
  activity: OverlordActivity;
  progress: number | null;
  tip: string;
}

interface PresenceInput {
  status: OverlordStatus | null;
  now: number;
}

export type { PresenceInput, PresenceLook };
