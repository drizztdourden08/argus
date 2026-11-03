/* @layer renderer-app @kind types */
import type { Collections } from '../stream/stream.type';

interface ScopeState {
  collections: Collections;
  seq: number;
  seqs: Map<string, number>;
  synced: Set<string>;
}

export type { ScopeState };
