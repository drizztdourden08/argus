/* @layer renderer-app @kind constants */
import type { CollectionState } from './stream.type';

const EMPTY_COLLECTION: CollectionState = { ready: false, docs: new Map() };

export { EMPTY_COLLECTION };
