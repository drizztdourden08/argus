/* @layer renderer-app @kind logic */
const rangeOf = (order: readonly string[], anchor: string | null, id: string): string[] => {
  const from = anchor === null ? -1 : order.indexOf(anchor);
  const to = order.indexOf(id);
  if (from < 0 || to < 0) return [id];
  return order.slice(Math.min(from, to), Math.max(from, to) + 1);
};

export { rangeOf };
