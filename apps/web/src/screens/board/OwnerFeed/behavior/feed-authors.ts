/* @layer renderer-app @kind logic */
import type { BoardWork } from '../../feed-model';
import type { Option } from '../../shared';
import { cardFacts } from './card-facts';

const feedAuthors = (work: BoardWork): Option[] => {
  const cards = [...work.waiting, ...work.leads, ...work.notices].map((card) => cardFacts(card).thread ?? '');
  const names = new Set([...cards, ...work.lines.map((line) => line.thread)]);
  return [...names].filter((name) => name !== '').sort().map((name) => ({ value: name, label: name }));
};

export { feedAuthors };
