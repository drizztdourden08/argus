/* @layer renderer-app @kind logic */
import { LAST_BOARD_KEY } from '../AppShell.constants';

const writeLastBoard = (key: string): void => {
  try {
    window.localStorage.setItem(LAST_BOARD_KEY, key);
  } catch {
    return;
  }
};

export { writeLastBoard };
