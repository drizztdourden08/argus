/* @layer renderer-app @kind logic */
import type { HandoverDoc } from '../../../../../data/types';
import { READY_STATES } from '../switch.constants';
import type { ReadyCount } from './ready-count.type';

const readyCount = (handovers: readonly HandoverDoc[]): ReadyCount => ({
  ready: handovers.filter((doc) => doc.state !== undefined && READY_STATES.has(doc.state)).length,
  total: handovers.length,
});

export { readyCount };
