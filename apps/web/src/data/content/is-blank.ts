/* @layer renderer-app @kind logic */
const isBlank = (value: unknown): boolean => {
  if (value === null || value === undefined) return true;
  if (typeof value === 'string') return value.trim() === '';
  return Array.isArray(value) && value.length === 0;
};

export { isBlank };
