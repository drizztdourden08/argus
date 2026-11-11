/* @layer renderer-app @kind logic */
import type { SyncStep } from '../compare.type';

const orderedSteps = (steps: readonly SyncStep[]): SyncStep[] => [...steps].sort((a, b) => a.id.localeCompare(b.id));

export { orderedSteps };
