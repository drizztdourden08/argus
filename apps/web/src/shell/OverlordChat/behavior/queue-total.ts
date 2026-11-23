/* @layer renderer-app @kind logic */
import type { OverlordQueue } from '../../../data/overlord';

const queueTotal = (queue: OverlordQueue): number => queue.wakes.length + queue.held.length + queue.alerts.length;

export { queueTotal };
