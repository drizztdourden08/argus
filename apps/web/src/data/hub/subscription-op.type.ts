/* @layer renderer-app @kind types */
type SubscriptionOp =
  | { op: 'sub'; scope: string; colls: string[]; since?: number }
  | { op: 'unsub'; scope: string; colls: string[] }
  | { op: 'presence'; scope: string; thread: string; tab: string }
  | { op: 'unpresence'; scope: string; tab: string }
  | { op: 'tail'; sub: string; key: string; rid: string; after: number }
  | { op: 'untail'; sub: string };

export type { SubscriptionOp };
