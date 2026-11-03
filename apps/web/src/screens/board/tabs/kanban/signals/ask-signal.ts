/* @layer renderer-app @kind logic */
import { ATTENTION_OF, askIds } from '../attention';
import type { AskKind, CardData } from '../model';
import { OPEN_ITEM } from './signal.constants';
import type { CardSignal } from './signal.type';

const askLabel = (kind: AskKind, ids: readonly string[]): string => {
  const one = ids.length === 1;
  const list = ids.join(', ');
  if (kind === 'question') return `${list} ${one ? 'is an open question' : 'are open questions'}, waiting for an answer`;
  return `${list} ${one ? 'is a decision' : 'are decisions'} to pick up`;
};

const askSignal = (kind: AskKind, card: CardData): CardSignal | null => {
  const ids = askIds(card.item, card.asks, kind);
  const [only] = ids;
  if (only === undefined) return null;
  const { icon, color } = ATTENTION_OF[kind];
  return { id: kind, icon, count: ids.length, label: askLabel(kind, ids), tone: color, opens: ids.length === 1 ? { kind: 'entry', id: only } : OPEN_ITEM };
};

export { askSignal };
