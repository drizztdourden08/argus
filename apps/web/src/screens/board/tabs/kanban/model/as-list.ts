/* @layer renderer-app @kind logic */
// A mirror field holds one entry on a board written before the lists, and a list of entries since.
const asList = <T>(value: T | T[] | null | undefined): T[] => {
  if (value === undefined || value === null) return [];
  return Array.isArray(value) ? value : [value];
};

export { asList };
