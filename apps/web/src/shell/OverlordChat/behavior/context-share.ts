/* @layer renderer-app @kind logic */
import type { OverlordUsage } from '../../../data/overlord';

const contextShare = (usage: OverlordUsage | null): number => {
  if (!usage || usage.context.windowTokens <= 0) return 0;
  return Math.min(1, Math.max(0, usage.context.usedTokens / usage.context.windowTokens));
};

export { contextShare };
