/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ProjectThread } from '../../../data/agents';
import type { ItemRef, SoftKind } from '../../../data/kanban';
import type { WithId } from '../../../data/react';

type ThreadDoc = WithId<ProjectThread>;

interface ThreadCounts {
  questions: number;
  toPickUp: number;
}

interface ThreadView {
  thread: ThreadDoc | null;
  work: ItemRef[];
  counts: ThreadCounts;
  paused: boolean;
}

interface ThreadOps {
  isBusy: (key: string) => boolean;
  setPaused: (thread: ThreadDoc, paused: boolean) => void;
  setField: (thread: ThreadDoc, field: 'subject' | 'brief', value: string) => void;
  soft: (kind: SoftKind, thread: ThreadDoc) => void;
}

interface ThreadDrawerProps {
  boardKey: string;
  id: string | null;
  onClose: () => void;
  onPurge: (thread: string) => void;
}

interface ThreadBodyProps {
  view: ThreadView & { thread: ThreadDoc };
  ops: ThreadOps;
  wide: boolean;
  onWide: () => void;
  onPurge: (thread: string) => void;
  onOpenItem: (id: string) => void;
}

interface ThreadToolsProps {
  thread: ThreadDoc;
  paused: boolean;
  ops: ThreadOps;
  wide: boolean;
  onWide: () => void;
  onPurge: (thread: string) => void;
}

interface ThreadState {
  coordinator: boolean;
  archived: boolean;
  live: boolean;
}

interface ThreadLiveToolsProps {
  thread: ThreadDoc;
  paused: boolean;
  ops: ThreadOps;
}

interface ThreadChatToolsProps {
  thread: ThreadDoc;
}

interface ThreadEndToolsProps {
  thread: ThreadDoc;
  state: ThreadState;
  ops: ThreadOps;
  onPurge: (thread: string) => void;
}

interface ThreadFactsProps {
  thread: ThreadDoc;
  counts: ThreadCounts;
  ops: ThreadOps;
}

interface ThreadAgentsProps {
  agents: NonNullable<ProjectThread['agents']>;
}

interface ThreadWorkProps {
  work: readonly ItemRef[];
  onOpen: (id: string) => void;
}

interface ThreadBriefProps {
  thread: ThreadDoc;
  ops: ThreadOps;
}

interface ThreadSectionProps {
  title: string;
  count?: number;
  children: ReactNode;
}

export type {
  ThreadAgentsProps, ThreadBodyProps, ThreadBriefProps, ThreadChatToolsProps, ThreadCounts, ThreadDoc, ThreadDrawerProps, ThreadEndToolsProps, ThreadFactsProps, ThreadLiveToolsProps, ThreadOps,
  ThreadSectionProps, ThreadState, ThreadToolsProps, ThreadView, ThreadWorkProps,
};
