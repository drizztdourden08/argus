/* @layer renderer-app @kind logic */
import { HUMAN } from './model.constants';
import type { EntryDocs } from './model.type';

const repliedTo = (entries: EntryDocs, kinds: readonly string[]): Set<string> =>
  new Set(entries.filter((entry) => entry.author === HUMAN && kinds.includes(entry.kind) && entry.replyTo).map((entry) => String(entry.replyTo)));

export { repliedTo };
