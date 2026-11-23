/* @layer renderer-app @kind logic */
import type { AccountDoc, AccountLimit } from '../../../../data/types';

const usageOf = (account: AccountDoc, kind: string): AccountLimit | null =>
  (account.limits ?? []).find((limit) => limit.kind === kind) ?? null;

export { usageOf };
