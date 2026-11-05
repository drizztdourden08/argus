/* @layer renderer-app @kind logic */
import type { MouseEvent } from 'react';
import { INTERACTIVE_SELECTOR } from './activate-on-body.constants';

const activateOnBody = (action: () => void) => (event: MouseEvent<HTMLElement>): void => {
  if (event.target instanceof Element && event.target.closest(INTERACTIVE_SELECTOR)) return;
  action();
};

export { activateOnBody };
