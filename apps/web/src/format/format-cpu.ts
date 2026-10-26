/* @layer renderer-app @kind logic */
import { asNumber } from './as-number';
import { ABSENT, CPU_DECIMALS_BELOW } from './format.constants';

const formatCpu = (percent: unknown): string => {
  const value = asNumber(percent);
  if (value === null) return ABSENT;
  return `${value < CPU_DECIMALS_BELOW ? value.toFixed(1) : String(Math.round(value))}%`;
};

export { formatCpu };
