/* @layer renderer-app @kind test */
import type { StallDoc } from '../../../src/data/stalls';
import { minutesAgo } from './fixtures';

const stall = (id: string, extra: Partial<StallDoc> = {}): StallDoc => ({
  id,
  project: 'rotp',
  lead: 'T-1',
  kind: 'idle',
  subject: 'T-1',
  text: 'stall: rotp idle 12 min, 3 open items without a blocker (I-4, I-7, I-9)',
  since: minutesAgo(12),
  raisedAt: minutesAgo(1),
  ackedAt: null,
  ackedBy: null,
  ackNote: null,
  ...extra,
});

const STALLS: Record<string, StallDoc> = {
  'rotp:idle:T-1:1': stall('rotp:idle:T-1:1'),
  'rotp:unread:W-9:2': stall('rotp:unread:W-9:2', { kind: 'unread', subject: 'W-9', since: minutesAgo(40), text: 'stall: rotp did not read D-11 for 10 min (it is in a turn)' }),
  'argus:silent:I-12:3': stall('argus:silent:I-12:3', {
    project: 'argus', kind: 'silent', subject: 'I-12', since: minutesAgo(70), text: 'stall: argus/K-12 doing 60 min, no agent', ackedAt: minutesAgo(5), ackedBy: 'portfolio/T-1', ackNote: 'asked the lead',
  }),
};

export { STALLS, stall };
