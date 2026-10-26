/* @layer renderer-app @kind constants */
import type { ListItemShape } from '@drizztdourden08/tessera';

const ROW_SHAPE: ListItemShape = { icon: false, columns: 4, action: true };

const CHAT_SHAPE: ListItemShape = { icon: false, columns: 2, action: true };

const NOW_TICK_MS = 15_000;

const FILTER_LABEL = 'Show agents';

const AGENT_ANCHOR = /^A-\d+$/;

const AGENTS_HASH = 'agents';

export { AGENT_ANCHOR, AGENTS_HASH, CHAT_SHAPE, FILTER_LABEL, NOW_TICK_MS, ROW_SHAPE };
