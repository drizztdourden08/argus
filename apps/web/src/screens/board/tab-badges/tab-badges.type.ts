/* @layer renderer-app @kind types */
import type { TabBadgeTone } from '@drizztdourden08/tessera';

interface Badge {
  count: number;
  tone?: TabBadgeTone;
}

type BoardBadges = Readonly<Record<string, Badge | undefined>>;

export type { Badge, BoardBadges };
