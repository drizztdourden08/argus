/* @layer renderer-app @kind logic */
import { STRIP_SELECTOR } from '../run.constants';
import type { RunKeyHandlers } from '../run.type';
import { keyIgnored } from './key-ignored';
import { keyStatus } from './key-status';
import { keyStep } from './key-step';

const handleKey = (event: KeyboardEvent, handlers: RunKeyHandlers): void => {
  const target = event.target instanceof Element ? event.target : null;
  if (keyIgnored(event, target)) return;
  const by = keyStep(event.key);
  const status = keyStatus(event.key);
  if (by !== null && !target?.closest(STRIP_SELECTOR)) handlers.onStep(by);
  else if (status) handlers.onRecord(status);
};

export { handleKey };
