/* @layer renderer-app @kind logic */
import { toMillis } from '../../../../format';
import { STALE_UPDATE_MS } from '../BoardHeader.constants';
import type { BoardHealth, HealthInput } from '../BoardHeader.type';

const healthOf = (input: HealthInput): BoardHealth => {
  const { lead, faults, offline, updatedAt, now } = input;
  if (offline) return { tone: 'danger', reason: 'The dashboard is unreachable.' };
  if (!lead) return { tone: 'danger', reason: 'No lead is registered on this board.' };
  if (faults > 0) return { tone: 'danger', reason: `${faults} registered thread${faults > 1 ? 's are' : ' is'} in fault.` };
  if (lead.status !== 'active' && lead.status !== 'paused') return { tone: 'danger', reason: `The lead is ${lead.status}.` };
  if (lead.status === 'paused') return { tone: 'warning', reason: 'The lead is paused.' };
  if (lead.activity === 'stalled') return { tone: 'warning', reason: 'The lead looks stalled.' };
  const at = toMillis(updatedAt);
  if (Number.isFinite(at) && now - at > STALE_UPDATE_MS) return { tone: 'warning', reason: 'The board has not updated for over an hour.' };
  return { tone: 'success', reason: 'The lead is active and the board is up to date.' };
};

export { healthOf };
