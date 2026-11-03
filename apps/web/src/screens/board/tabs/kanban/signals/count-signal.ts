/* @layer renderer-app @kind logic */
import { ATTENTION_OF } from '../attention';
import type { AttentionKind } from '../attention';
import { OPEN_ITEM, PLURAL_TEXT } from './signal.constants';
import type { CardSignal } from './signal.type';

const countSignal = (kind: AttentionKind, count: number): CardSignal | null => {
  const words = PLURAL_TEXT[kind];
  if (count === 0 || !words) return null;
  const { icon, color } = ATTENTION_OF[kind];
  return { id: kind, icon, count, label: `${count} ${words[count === 1 ? 0 : 1]}`, tone: color, opens: OPEN_ITEM };
};

export { countSignal };
