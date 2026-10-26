/* @layer renderer-app @kind logic */
import type { WakeDoc } from './model.type';

const latestWakes = (wakes: readonly WakeDoc[]): Map<string, WakeDoc> => {
  const latest = new Map<string, WakeDoc>();
  for (const wake of wakes) {
    const current = wake.subject ? latest.get(wake.subject) : undefined;
    if (wake.subject && (!current || String(wake.at ?? '') > String(current.at ?? ''))) latest.set(wake.subject, wake);
  }
  return latest;
};

export { latestWakes };
