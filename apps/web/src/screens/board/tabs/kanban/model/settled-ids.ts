/* @layer renderer-app @kind logic */
import { ANSWER_KINDS } from './model.constants';
import { repliedTo } from './replied-to';
import type { EntryDocs } from './model.type';

const settledIds = (entries: EntryDocs): Set<string> => new Set([...repliedTo(entries, ANSWER_KINDS), ...repliedTo(entries, ['ack'])]);

export { settledIds };
