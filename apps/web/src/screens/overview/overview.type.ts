/* @layer renderer-app @kind types */
import type { TabBadgeTone, TabItem } from '@drizztdourden08/tessera';
import type { StreamStatus } from '../../data/stream';
import type { BoardEntries, BoardEntry, OverviewView } from './boards/boards-view.type';
import type { SendDown } from './dashboards/dashboards.type';

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

interface TabBodyProps {
  tab: string;
  data: BoardEntries;
  view: OverviewView;
  send: SendDown;
  wide: boolean;
  badges: TabBadges;
}

interface OverviewHeaderProps {
  tab: string;
  data: BoardEntries;
  view: OverviewView;
  status: StreamStatus;
  send: SendDown;
  wide: boolean;
}

interface OverviewAlertsProps {
  entries: readonly BoardEntry[];
  status: StreamStatus;
}

interface OverviewTabsProps {
  tabs: TabItem[];
  activeTab: string;
  onTabChange: (tab: string) => void;
}

export type { ChipFilterProps, OverviewAlertsProps, OverviewHeaderProps, OverviewTabsProps, TabBadges, TabBodyProps };
