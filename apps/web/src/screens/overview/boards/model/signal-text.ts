/* @layer renderer-app @kind logic */
import type { SignalId } from './board.type';
import { COUNTED, FIXED } from './signal-text.constants';

const signalText = (id: SignalId, count: number): string => FIXED[id] ?? COUNTED[id]?.(count) ?? '';

export { signalText };
