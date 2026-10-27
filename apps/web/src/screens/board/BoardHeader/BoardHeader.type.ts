/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ProjectThread } from '../../../data/agents';
import type { WithId } from '../../../data/react';
import type { StreamStatus } from '../../../data/stream';

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

export type { BoardHeaderProps, BoardStampProps, LeadStatusProps };
