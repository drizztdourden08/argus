/* @layer renderer-app @kind logic */
import type { KeyboardEvent } from 'react';
import { OUTCOMES, TYPING_SELECTOR } from '../tests.constants';
import type { Outcome, RowMenu } from '../tests.type';

const outcomeKey = (event: KeyboardEvent<HTMLElement>): number => {
  if (event.altKey || event.ctrlKey || event.metaKey) return -1;
  if (event.target instanceof Element && (event.target.matches(TYPING_SELECTOR) || event.target.closest('[role="menu"]'))) return -1;
  return /^[1-4]$/.test(event.key) ? Number(event.key) - 1 : -1;
};

const rowKeys = (toRun: boolean, controls: Outcome, menu: RowMenu) => (event: KeyboardEvent<HTMLElement>): void => {
  menu.onKeyDown(event);
  const outcome = OUTCOMES[outcomeKey(event)];
  if (!toRun || !outcome || controls.busy) return;
  event.preventDefault();
  controls.choose(outcome.id);
};

export { rowKeys };
