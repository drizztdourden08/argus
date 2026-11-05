/* @layer renderer-app @kind logic */
import type { KeyboardEvent } from 'react';
import { ownsEvent } from './card-click';

const cardKey = (open: () => void) => (event: KeyboardEvent) => {
  if ((event.key !== 'Enter' && event.key !== ' ') || ownsEvent(event)) return;
  event.preventDefault();
  open();
};

export { cardKey };
