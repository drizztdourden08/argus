/* @layer renderer-app @kind logic */
import type { Entry } from '../../../../data/types';
import type { AskMode } from '../../feed-model';
import type { AskCopy } from '../OwnerFeed.type';

const askCopy = (entry: Entry, mode: AskMode): AskCopy => {
  if (mode === 'decide') return { kind: 'decision', label: 'Or write your own answer', verb: 'Decide' };
  if (entry.kind === 'message') return { kind: 'followup', label: `Reply to ${entry.thread}.`, verb: 'Reply' };
  return { kind: 'followup', label: `Answer ${entry.thread}.`, verb: 'Answer' };
};

export { askCopy };
