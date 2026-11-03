/* @layer renderer-app @kind logic */
const asList = <T>(value: T | T[] | null | undefined): T[] => {
  if (value === undefined || value === null) return [];
  return Array.isArray(value) ? value : [value];
};

export { asList };
