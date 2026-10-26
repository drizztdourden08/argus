/* @layer renderer-app @kind logic */
import { asNumber } from '../../format';

const killedCount = (result: { killed?: unknown } | null | undefined, fallback: number): number => {
  const killed = result?.killed;
  if (Array.isArray(killed)) return killed.length;
  return asNumber(killed) ?? fallback;
};

export { killedCount };
