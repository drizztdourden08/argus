/* @layer renderer-app @kind logic */
import { LAST_BOARD_KEY } from '../AppShell.constants';

const readLastBoard = (): string | null => {
  try {
    return window.localStorage.getItem(LAST_BOARD_KEY);
  } catch {
    return null;
  }
};

export { readLastBoard };
