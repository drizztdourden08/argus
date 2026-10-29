/* @layer renderer-app @kind logic */
import { asNumber } from './as-number';
import { ABSENT, BYTE_UNITS, BYTES_PER_UNIT, WHOLE_FROM } from './format.constants';

const formatBytes = (bytes: unknown): string => {
  const value = asNumber(bytes);
  if (value === null) return ABSENT;
  let unit = 0;
  let scaled = value;
  while (scaled >= BYTES_PER_UNIT && unit < BYTE_UNITS.length - 1) {
    scaled /= BYTES_PER_UNIT;
    unit += 1;
  }
  const text = unit === 0 || scaled >= WHOLE_FROM ? Math.round(scaled) : scaled.toFixed(1);
  return `${text} ${BYTE_UNITS[unit]}`;
};

export { formatBytes };
