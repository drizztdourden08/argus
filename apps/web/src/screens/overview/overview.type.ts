/* @layer renderer-app @kind types */
import type { StreamStatus } from '../../data/stream';
import type { Card } from '../../data/types';

interface TabBadges {
  counts: Record<string, number>;
  freshActivity: number;
  freshNotifications: number;
}

interface TabBodyProps {
  tab: string;
  cards: Card[];
  status: StreamStatus;
  badges: TabBadges;
}

export type { TabBadges, TabBodyProps };
