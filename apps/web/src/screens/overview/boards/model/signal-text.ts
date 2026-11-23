/* @layer renderer-app @kind logic */
import type { SignalId } from './board.type';

const plural = (count: number, one: string, many: string): string => (count === 1 ? one : many);

const COUNTED: Partial<Record<SignalId, (count: number) => string>> = {
  'needs-you': (count) => plural(count, '1 needs you', `${count} need you`),
  failing: (count) => plural(count, '1 failing test', `${count} failing tests`),
  blocked: (count) => `${count} blocked`,
  stalled: (count) => `${count} stalled`,
  paused: (count) => `${count} paused`,
};

const FIXED: Partial<Record<SignalId, string>> = {
  'sync-due': 'Sync due',
  'master-missing': 'Master missing',
  archived: 'Archived',
};

const signalText = (id: SignalId, count: number): string => FIXED[id] ?? COUNTED[id]?.(count) ?? '';

export { signalText };
