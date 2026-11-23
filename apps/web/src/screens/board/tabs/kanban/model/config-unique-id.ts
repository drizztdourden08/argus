/* @layer renderer-app @kind logic */
const configUniqueId = (taken: readonly string[], base: string): string => {
  const free = (n: number): string => (n === 1 ? base : `${base}-${n}`);
  let n = 1;
  while (taken.includes(free(n))) n += 1;
  return free(n);
};

export { configUniqueId };
