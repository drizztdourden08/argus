/* @layer renderer-app @kind logic */
import type { Entry } from '../../../data/types';
import type { OwnerMarks } from './feed.type';
import { isUnapplied } from './is-unapplied';

const answeredBy = (entries: Entry[], kinds: string[], pendingOnly: boolean): Set<string> => {
  const targets = entries
    .filter((entry) => entry.author === 'human' && kinds.includes(entry.kind) && entry.replyTo)
    .filter((entry) => !pendingOnly || isUnapplied(entry))
    .map((entry) => entry.replyTo as string);
  return new Set(targets);
};

const ownerMarks = (entries: Entry[]): OwnerMarks => ({
  acked: answeredBy(entries, ['ack'], false),
  answered: answeredBy(entries, ['followup', 'decision'], false),
  done: answeredBy(entries, ['done'], true),
});

export { ownerMarks };
