/* @layer renderer-app @kind logic */
import { requiresParent } from '../../../../../data/registry';
import { typeOf } from '../model';
import type { CardData } from '../model';
import { OPEN_ITEM } from './signal.constants';
import type { CardSignal } from './signal.type';

const noEpicSignal = (card: CardData): CardSignal | null =>
  !card.item.parent && !card.item.archivedAt && requiresParent(typeOf(card.item))
    ? { id: 'no-epic', icon: 'triangle-alert', label: 'no epic', tone: 'warning', opens: OPEN_ITEM }
    : null;

export { noEpicSignal };
