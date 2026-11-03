/* @layer renderer-app @kind types */
import type { DocData } from '../api';
import type { TabLink } from '../hub/tab-link';
import type { ScopeStream } from './scope-stream';

type Listener = () => void;

type StreamStatus = 'idle' | 'connecting' | 'live' | 'unavailable' | 'unauthorized';

interface CollectionState {
  ready: boolean;
  docs: ReadonlyMap<string, DocData>;
}

type Collections = ReadonlyMap<string, CollectionState>;

interface SourceLike {
  addEventListener(type: string, listener: (event: MessageEvent<string>) => void): void;
  close(): void;
}

interface ScopeStreams {
  link: TabLink;
  get(scope: string): ScopeStream;
}

interface WireDoc {
  id: string;
  data?: DocData;
  [field: string]: unknown;
}

interface SnapshotEvent {
  scope: string;
  seq: number;
  coll: string;
  docs: WireDoc[];
}

interface WireChange {
  coll: string;
  id: string;
  op: 'set' | 'delete';
  data?: DocData;
}

interface ChangesEvent {
  scope: string;
  seq: number;
  changes: WireChange[];
}

interface AppliedChanges {
  collections: Collections;
  touched: string[];
}

export type {
  AppliedChanges,
  ChangesEvent,
  CollectionState,
  Collections,
  Listener,
  ScopeStreams,
  SnapshotEvent,
  SourceLike,
  StreamStatus,
  WireChange,
  WireDoc,
};
