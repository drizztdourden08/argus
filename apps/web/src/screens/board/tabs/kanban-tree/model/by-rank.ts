/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';

const byRank = (items: readonly ItemRef[]): ItemRef[] => [...items].sort((a, b) => (a.rank ?? 0) - (b.rank ?? 0));

export { byRank };
