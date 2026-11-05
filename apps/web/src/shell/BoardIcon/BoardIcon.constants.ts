/* @layer renderer-app @kind constants */
import type { BoardIconSize } from './BoardIcon.type';

const BOARD_ICON_PX: Readonly<Record<BoardIconSize, number>> = { sm: 18, md: 28, lg: 48 };

export { BOARD_ICON_PX };
