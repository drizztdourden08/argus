/* @layer tooling-scripts @kind logic */
const filtersOf = (occurrence) => {
  const prefix = occurrence.modPath ? `${occurrence.modPath}::` : '';
  if (occurrence.inline.length > 0) return occurrence.inline.map((name) => `${prefix}${name}::`);
  return prefix ? [prefix] : null;
};

export { filtersOf };
