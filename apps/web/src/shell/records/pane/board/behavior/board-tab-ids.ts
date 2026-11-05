/* @layer renderer-app @kind logic */
import { kindOf } from '../../../../../data/registry';
import { BOARD_KIND } from '../board.constants';
import { BOARD_TABS } from '../board-tabs.constants';

const boardTabIds = (): string[] => (kindOf(BOARD_KIND)?.tabs ?? []).filter((id) => Object.hasOwn(BOARD_TABS, id));

export { boardTabIds };
