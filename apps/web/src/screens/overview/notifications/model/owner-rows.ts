/* @layer renderer-app @kind logic */
import type { Card } from '../../../../data/types';
import type { OwnerRow } from './model.type';

const byProjectThenId = (a: OwnerRow, b: OwnerRow): number =>
  a.name.localeCompare(b.name) || a.id.localeCompare(b.id, undefined, { numeric: true });

const ownerRows = (cards: readonly Card[]): OwnerRow[] =>
  cards
    .flatMap((card) => (card.ownerItems ?? []).map((item) => ({ ...item, project: card.key, name: card.name })))
    .sort(byProjectThenId);

export { ownerRows };
