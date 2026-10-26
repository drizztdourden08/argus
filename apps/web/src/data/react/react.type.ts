/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ScopeStreams, StreamStatus } from '../stream';
import type { BoardMeta, Card, Thread } from '../types';

interface DataProviderProps {
  streams?: ScopeStreams;
  children: ReactNode;
}

type WithId<T> = T & { id: string };

interface CollectionResult<T> {
  ready: boolean;
  docs: WithId<T>[];
}

interface DocLookup<T> {
  ready: boolean;
  doc: WithId<T> | null;
}

interface BoardView {
  ready: boolean;
  status: StreamStatus;
  name: string;
  meta: BoardMeta | null;
  threads: Thread[];
}

interface PortfolioView {
  ready: boolean;
  status: StreamStatus;
  cards: Card[];
}

export type { BoardView, CollectionResult, DataProviderProps, DocLookup, PortfolioView, WithId };
