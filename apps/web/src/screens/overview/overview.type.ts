/* @layer renderer-app @kind types */
import type { TabBadgeTone } from '@drizztdourden08/tessera';
import type { StreamStatus } from '../../data/stream';
import type { Card } from '../../data/types';

interface TabBadges {
  counts: Record<string, number>;
  tones: Record<string, TabBadgeTone>;
  freshActivity: number;
  freshNotifications: number;
}

interface ShortAgeProps {
  value: string | number;
  ago?: boolean;
}

interface ChipFilterProps {
  label: string;
  value: string;
  all: string;
  options: readonly { value: string; label: string }[];
  onChange: (value: string) => void;
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

export type { ChipFilterProps, ProjectSwitcherProps, ShortAgeProps, TabBadges, TabBodyProps };
