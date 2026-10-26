/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { StreamStatus } from '../data/stream';

interface PageFrameProps {
  children: ReactNode;
}

interface StatusNoticeProps {
  status: StreamStatus;
}

interface HashTab {
  tab: string;
  select: (tab: string) => void;
}

export type { HashTab, PageFrameProps, StatusNoticeProps };
