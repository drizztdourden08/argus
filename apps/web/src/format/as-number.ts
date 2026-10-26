/* @layer renderer-app @kind logic */
const asNumber = (value: unknown): number | null =>
  typeof value === 'number' && Number.isFinite(value) ? value : null;

export { asNumber };
