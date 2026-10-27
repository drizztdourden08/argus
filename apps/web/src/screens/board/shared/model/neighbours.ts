/* @layer renderer-app @kind logic */
const neighbours = (order: readonly string[], id: string): { previous: string | null; next: string | null } => {
  const at = order.indexOf(id);
  if (at < 0) return { previous: null, next: null };
  return { previous: order[at - 1] ?? null, next: order[at + 1] ?? null };
};

export { neighbours };
