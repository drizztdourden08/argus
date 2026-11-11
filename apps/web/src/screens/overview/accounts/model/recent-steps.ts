/* @layer renderer-app @kind logic */
import { LOG_LIMIT } from '../compare.constants';
import type { SyncStep } from '../compare.type';

const orderedSteps = (steps: readonly SyncStep[]): SyncStep[] => [...steps].sort((a, b) => a.id.localeCompare(b.id));

const recentSteps = (steps: readonly SyncStep[]): SyncStep[] => steps.slice(-LOG_LIMIT).reverse();

export { orderedSteps, recentSteps };
