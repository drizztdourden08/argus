/* @layer renderer-app @kind logic */
import { atLimitText, SWITCH_OFF_TEXT, SWITCH_RUNNING_TEXT } from '../accounts.constants';
import type { RowState } from '../accounts.type';

const blockReason = (state: RowState, switching: boolean): string => {
  if (state.full) return `${atLimitText(state.until)}.`;
  if (!state.receive) return SWITCH_OFF_TEXT;
  return switching ? SWITCH_RUNNING_TEXT : '';
};

export { blockReason };
