/* @layer renderer-app @kind types */
import type { ReactNode, RefObject } from 'react';
import type { ProjectThread } from '../../../data/agents';
import type { WithId } from '../../../data/react';
import type { StreamStatus } from '../../../data/stream';
import type { Wake } from '../../../data/types';

interface BoardHeaderProps {
  boardKey: string;
  name: string;
  status: StreamStatus;
  updatedAt: string | null;
  tools: ReactNode;
}

interface LeadStatusProps {
  lead: WithId<ProjectThread> | null;
}

interface BoardStampProps {
  status: StreamStatus;
  updatedAt: string | null;
}

interface BoardWorktreesProps {
  boardKey: string;
}

interface BoardNoticesProps {
  boardKey: string;
}

interface NoticeListProps {
  notices: readonly WithId<Wake>[];
}

interface NoticesView {
  notices: WithId<Wake>[];
  anchor: RefObject<HTMLButtonElement | null>;
  open: boolean;
  toggle: () => void;
  close: () => void;
}

export type { BoardHeaderProps, BoardNoticesProps, BoardStampProps, BoardWorktreesProps, LeadStatusProps, NoticeListProps, NoticesView };
