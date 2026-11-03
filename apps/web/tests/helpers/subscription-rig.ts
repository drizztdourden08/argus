/* @layer renderer-app @kind test */
import type { MuxOp } from '../../src/data/hub/hub.type';
import { SubscriptionTable } from '../../src/data/hub/subscription-table';

const LINGER_MS = 10_000;

const created = (lingerMs?: number) => {
  const expired: MuxOp[][] = [];
  const table = new SubscriptionTable({ onExpire: (ops) => expired.push(ops), lingerMs });
  return { table, expired };
};

const wantAndFlush = (table: SubscriptionTable, coll = 'items'): void => {
  table.want('a', 'rotp', coll);
  table.flush();
};

const unwantAndFlush = (table: SubscriptionTable, coll = 'items'): void => {
  table.unwant('a', 'rotp', coll);
  table.flush();
};

export { created, LINGER_MS, unwantAndFlush, wantAndFlush };
