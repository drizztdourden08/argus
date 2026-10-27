/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { StreamStatus } from '../data/stream';

interface PageFrameProps {
  children: ReactNode;
}

interface StatusNoticeProps {
  status: StreamStatus;
}

interface RegisteredFrameProps {
  label: string;
  rows: readonly { state: string }[];
  children: ReactNode;
}

interface PanelProps {
  title: ReactNode;
  note?: ReactNode;
  tools?: ReactNode;
  children: ReactNode;
}

interface ProjectSwitcherProps {
  current?: string | null;
}

interface HashTab {
  tab: string;
  select: (tab: string) => void;
}

export type { HashTab, PageFrameProps, PanelProps, ProjectSwitcherProps, RegisteredFrameProps, StatusNoticeProps };
