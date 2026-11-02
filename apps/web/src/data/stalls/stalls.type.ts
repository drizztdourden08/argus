/* @layer renderer-app @kind types */
type StallKind = 'unread' | 'unstarted' | 'idle' | 'silent';

interface StallDoc {
  id: string;
  project: string;
  lead: string | null;
  kind: StallKind;
  subject: string;
  text: string;
  since: string;
  raisedAt: string;
  ackedAt: string | null;
  ackedBy: string | null;
  ackNote: string | null;
}

export type { StallDoc };
