/* @layer renderer-app @kind logic */
import type { TagCategoryColor } from '@drizztdourden08/tessera';
import { TYPE_TINTS } from './card-tint.constants';
import type { CardData } from './model.type';
import { typeOf } from './type-of';

const cardTint = (card: CardData): TagCategoryColor | undefined => TYPE_TINTS[typeOf(card.item)];

export { cardTint };
