/* @layer renderer-app @kind constants */
import type { BoardLive } from './board.type';

const NO_LIVE: BoardLive = { fault: null, stalls: 0, syncDue: false };

export { NO_LIVE };
