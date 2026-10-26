/* @layer renderer-app @kind constants */
import type { ListItemShape } from '@drizztdourden08/tessera';

const ROW_SHAPE: ListItemShape = { icon: false, columns: 4, action: true };

const CHAT_SHAPE: ListItemShape = { icon: false, columns: 2, action: true };

const NOW_TICK_MS = 15_000;

const FILTER_LABEL = 'Show agents';

export { CHAT_SHAPE, FILTER_LABEL, NOW_TICK_MS, ROW_SHAPE };
