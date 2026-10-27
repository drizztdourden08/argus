/* @layer renderer-app @kind types */
import type { IconName, TabBadgeTone, TextProps } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { StreamStatus } from '../../data/stream';
import type { Card } from '../../data/types';

interface TabBadges {
  counts: Record<string, number>;
  tones: Record<string, TabBadgeTone>;
  freshActivity: number;
  freshNotifications: number;
}

interface ChipFilterProps {
  label: string;
  value: string;
  all: string;
  options: readonly { value: string; label: string }[];
  onChange: (value: string) => void;
}

interface FeedRowProps {
  icon: IconName;
  tone: TextProps['tone'];
  title?: string;
  at: string | null | undefined;
  children: ReactNode;
}

interface FeedTableProps {
  label: string;
  children: ReactNode;
}

interface OverviewHeaderProps {
  cards: Card[];
}

interface ProjectSwitcherProps {
  cards: Card[];
}

interface TabBodyProps {
  tab: string;
  cards: Card[];
  status: StreamStatus;
  badges: TabBadges;
}

export type { ChipFilterProps, FeedRowProps, FeedTableProps, OverviewHeaderProps, ProjectSwitcherProps, TabBadges, TabBodyProps };
