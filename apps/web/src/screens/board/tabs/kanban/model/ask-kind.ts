/* @layer renderer-app @kind logic */
import type { Entry } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import type { AskKind } from './model.type';

const askKind = (entry: WithId<Entry>, settled: ReadonlySet<string>): AskKind | null => {
  if (entry.kind === 'question' && entry.status === 'open' && !settled.has(entry.id)) return 'question';
  if (entry.kind === 'decision' && entry.status === 'pending') return 'decision';
  return null;
};

export { askKind };
