/* @layer renderer-app @kind logic */
import { goneThreads } from './gone-threads';
import type { PurgeCounts, PurgeCriteria, PurgeData, PurgeDoc } from './trash.type';

const inSet = (set: ReadonlySet<string>, key: string | null | undefined): boolean => key !== null && key !== undefined && set.has(key);

const withReplies = (entries: readonly PurgeDoc[], hit: Set<string>, others: readonly ReadonlySet<string>[]): Set<string> => {
  for (let again = true; again; ) {
    again = false;
    for (const entry of entries) {
      if (hit.has(entry.id) || !entry.replyTo) continue;
      if (hit.has(entry.replyTo) || others.some((set) => set.has(String(entry.replyTo)))) {
        hit.add(entry.id);
        again = true;
      }
    }
  }
  return hit;
};

const purgeCounts = (criteria: PurgeCriteria, data: PurgeData, now: number): PurgeCounts => {
  const gone = goneThreads(data.threads, criteria, now);
  const trashed = (doc: PurgeDoc): boolean => Boolean(criteria.trash && doc.deletedAt);
  const items = new Set(data.items.filter((item) => inSet(gone, item.thread) || trashed(item)).map((item) => item.id));
  const tests = new Set(data.tests.filter((test) => inSet(items, test.item) || inSet(gone, test.thread) || trashed(test)).map((test) => test.id));
  const actions = new Set(data.actions.filter((action) => inSet(gone, action.thread) || inSet(items, action.item) || trashed(action)).map((action) => action.id));
  const hit = new Set(data.entries.filter((entry) => inSet(gone, entry.thread) || inSet(items, entry.item) || trashed(entry)).map((entry) => entry.id));
  const entries = withReplies(data.entries, hit, [tests, actions, gone]);
  const logLines = data.log.reduce((total, doc) => total + doc.entries.filter((line) => inSet(gone, line.thread) || inSet(items, line.item)).length, 0);
  const ac = data.items.filter((item) => items.has(item.id)).reduce((total, item) => total + (item.ac?.length ?? 0), 0);
  return { threads: gone.size, items: items.size, ac, tests: tests.size, entries: entries.size, actions: actions.size, logLines };
};

export { purgeCounts };
