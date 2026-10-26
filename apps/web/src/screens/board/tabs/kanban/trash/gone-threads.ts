/* @layer renderer-app @kind logic */
import { MS_PER_DAY } from './trash.constants';
import type { PurgeCriteria, PurgeThread } from './trash.type';

const isIdle = (thread: PurgeThread, days: number | undefined, now: number): boolean =>
  days !== undefined && Boolean(thread.lastSeen) && now - Date.parse(String(thread.lastSeen)) > days * MS_PER_DAY;

const hasStatus = (thread: PurgeThread, status: readonly string[] | undefined): boolean =>
  status !== undefined && (status.includes(thread.status ?? '') || (status.includes('placeholder') && thread.placeholder === true));

const isListed = (thread: PurgeThread, criteria: PurgeCriteria): boolean =>
  criteria.threads?.includes(thread.id) === true || (criteria.keep !== undefined && !criteria.keep.includes(thread.id));

const matches = (thread: PurgeThread, criteria: PurgeCriteria, now: number): boolean =>
  isListed(thread, criteria) || hasStatus(thread, criteria.status) || isIdle(thread, criteria.idleDays, now);

const goneThreads = (threads: readonly PurgeThread[], criteria: PurgeCriteria, now: number): Set<string> => {
  const gone = new Set(threads.filter((thread) => thread.role !== 'coordinator' && matches(thread, criteria, now)).map((thread) => thread.id));
  for (const thread of threads) if (thread.placeholder && thread.mergedInto && gone.has(thread.mergedInto)) gone.add(thread.id);
  return gone;
};

export { goneThreads };
