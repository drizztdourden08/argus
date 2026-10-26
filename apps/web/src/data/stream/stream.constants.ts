/* @layer renderer-app @kind constants */
import type { CollectionState } from './stream.type';

const EMPTY_COLLECTION: CollectionState = { ready: false, docs: new Map() };

const FAILURES_BEFORE_UNAVAILABLE = 3;
const RECONNECT_BASE_MS = 500;
const RECONNECT_CAP_MS = 8000;
const SOURCE_EVENTS = ['snapshot', 'changes', 'ready', 'reset'] as const;

export { EMPTY_COLLECTION, FAILURES_BEFORE_UNAVAILABLE, RECONNECT_BASE_MS, RECONNECT_CAP_MS, SOURCE_EVENTS };
