/* @layer renderer-app @kind logic */
import type { Attention } from './board.type';

const cardTone = (attention: Attention): 'danger' | 'warning' | undefined => {
  if (attention.failing || attention.blocked) return 'danger';
  return attention.needs ? 'warning' : undefined;
};

export { cardTone };
