/* @layer renderer-app @kind logic */
import { BOARD_ICON_NAMES } from './board-icon.constants';
import type { BoardIconName } from './board-icon.type';

const toBoardIconName = (value: string): BoardIconName | undefined => BOARD_ICON_NAMES.find((candidate) => candidate === value);

export { toBoardIconName };
