/* @layer renderer-app @kind logic */
import type { Entry } from '../../../data/types';
import type { AskMode } from './feed.type';

const askMode = (entry: Entry): AskMode => {
  if (entry.kind !== 'question') return 'info';
  if (entry.options?.length) return 'decide';
  return entry.needs === 'decision' ? 'info' : 'answer';
};

export { askMode };
