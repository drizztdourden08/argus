/* @layer renderer-app @kind logic */
const toMillis = (value: unknown): number => {
  if (typeof value === 'number') return value;
  if (typeof value === 'string') return Date.parse(value);
  return Number.NaN;
};

export { toMillis };
