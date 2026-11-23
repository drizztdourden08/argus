/* @layer renderer-app @kind logic */
import { MAX_PERCENT, MIN_PERCENT } from '../accounts.constants';

const clampPercent = (value: number, fallback: number): number =>
  Number.isFinite(value) ? Math.min(MAX_PERCENT, Math.max(MIN_PERCENT, value)) : fallback;

export { clampPercent };
