/* @layer renderer-app @kind types */
interface KeyStatus {
  set: boolean;
  last4: string | null;
  savedAt: string | null;
}

interface OverlordKeyApi {
  status: KeyStatus | null;
  busy: boolean;
  problem: string | null;
  save: (key: string) => Promise<boolean>;
  remove: () => Promise<boolean>;
}

export type { KeyStatus, OverlordKeyApi };
