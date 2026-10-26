/* @layer renderer-app @kind logic */
const neighbour = (ids: readonly string[], current: string, step: number): string | null => {
  const at = ids.indexOf(current);
  return at < 0 ? null : (ids[at + step] ?? null);
};

export { neighbour };
