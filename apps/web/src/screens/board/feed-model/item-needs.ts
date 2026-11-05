/* @layer renderer-app @kind logic */
import type { Entry } from '../../../data/types';
import { NO_NEEDS } from './feed.constants';
import type { FeedCard, ItemNeeds, NeedCount } from './feed.type';

const bump = (needs: Map<string, NeedCount>, item: string | null | undefined, field: keyof NeedCount): void => {
  if (!item) return;
  const known = needs.get(item) ?? NO_NEEDS;
  needs.set(item, { ...known, [field]: known[field] + 1 });
};

const countCard = (needs: Map<string, NeedCount>, card: FeedCard): void => {
  if (card.type === 'action') bump(needs, card.data.item, 'steps');
  else if (card.type === 'test') bump(needs, card.data.item, 'tests');
  else if (card.type === 'run') bump(needs, card.data.plan, 'tests');
  else if (card.type === 'decision' || card.type === 'question') bump(needs, card.data.item, 'questions');
};

const itemNeeds = (cards: FeedCard[], entries: Entry[]): ItemNeeds => {
  const needs = new Map<string, NeedCount>();
  for (const card of cards) countCard(needs, card);
  for (const entry of entries) if (entry.kind === 'decision' && entry.status === 'pending') bump(needs, entry.item, 'decisions');
  return needs;
};

export { itemNeeds };
