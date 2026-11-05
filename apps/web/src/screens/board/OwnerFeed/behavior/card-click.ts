/* @layer renderer-app @kind logic */
import type { KeyboardEvent, MouseEvent } from 'react';
import { CARD_OWN_CONTROLS } from '../OwnerFeed.constants';

const ownsEvent = (event: MouseEvent | KeyboardEvent): boolean => {
  const target = event.target;
  return event.defaultPrevented || (target instanceof Element && target.closest(CARD_OWN_CONTROLS) !== null);
};

const cardClick = (open: () => void) => (event: MouseEvent) => {
  if (!ownsEvent(event)) open();
};

const cardKey = (open: () => void) => (event: KeyboardEvent) => {
  if ((event.key !== 'Enter' && event.key !== ' ') || ownsEvent(event)) return;
  event.preventDefault();
  open();
};

export { cardClick, cardKey };
