/* @layer renderer-app @kind logic */
import type { WithId } from '../react';
import type { Card } from '../types';
import type { BoardDoc, BoardItem } from './boards.type';

const byKey = (a: BoardItem, b: BoardItem): number => a.doc.key.localeCompare(b.doc.key);

const joinBoards = (docs: readonly WithId<BoardDoc>[], cards: readonly Card[]): BoardItem[] => {
  const cardOf = new Map(cards.map((card) => [card.key, card]));
  return docs.map((doc) => ({ doc, card: cardOf.get(doc.key) ?? null })).sort(byKey);
};

export { joinBoards };
