/* @layer renderer-app @kind logic */
import type { MouseEvent } from 'react';

const INTERACTIVE_SELECTOR = 'button, a, input, textarea, select, [role="menu"], [role="menuitem"], [role="button"], [contenteditable]';

const activateOnBody = (action: () => void) => (event: MouseEvent<HTMLElement>): void => {
  if (event.target instanceof Element && event.target.closest(INTERACTIVE_SELECTOR)) return;
  action();
};

export { activateOnBody };
