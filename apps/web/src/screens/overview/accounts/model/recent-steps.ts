/* @layer renderer-app @kind logic */
import { LOG_LIMIT } from '../compare.constants';
import type { SyncStep } from '../compare.type';

const recentSteps = (steps: readonly SyncStep[]): SyncStep[] => steps.slice(-LOG_LIMIT).reverse();

export { recentSteps };
