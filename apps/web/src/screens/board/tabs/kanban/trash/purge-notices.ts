/* @layer renderer-app @kind logic */
import type { Entry } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { HUMAN } from '../model';
import { PURGE_SETTLED } from './trash.constants';
import type { PurgeNotice } from './trash.type';

const purgeNotices = (entries: readonly WithId<Entry>[]): PurgeNotice[] => {
  const acked = new Set(entries.filter((entry) => entry.author === HUMAN && entry.kind === 'ack' && entry.replyTo).map((entry) => String(entry.replyTo)));
  return entries
    .filter((entry) => entry.author === HUMAN && entry.kind === 'purge' && PURGE_SETTLED.includes(String(entry.status)) && !acked.has(entry.id))
    .map((entry) => ({ id: entry.id, status: String(entry.status), outcome: entry.outcome ?? '' }));
};

export { purgeNotices };
