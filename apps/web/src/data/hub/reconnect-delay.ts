/* @layer renderer-app @kind logic */
import { RECONNECT_BASE_MS, RECONNECT_CAP_MS } from './hub.constants';

const reconnectDelay = (failures: number): number => Math.min(RECONNECT_BASE_MS * 2 ** (failures - 1), RECONNECT_CAP_MS);

export { reconnectDelay };
