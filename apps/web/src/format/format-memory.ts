/* @layer renderer-app @kind logic */
import { asNumber } from './as-number';
import { ABSENT, GB_WHOLE_FROM_MB, MB_PER_GB } from './format.constants';

const formatMemory = (megabytes: unknown): string => {
  const value = asNumber(megabytes);
  if (value === null) return ABSENT;
  if (value < MB_PER_GB) return `${Math.round(value)} MB`;
  return `${(value / MB_PER_GB).toFixed(value >= GB_WHOLE_FROM_MB ? 0 : 1)} GB`;
};

export { formatMemory };
