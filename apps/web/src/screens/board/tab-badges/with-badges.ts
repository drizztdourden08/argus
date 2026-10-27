/* @layer renderer-app @kind logic */
import type { TabItem } from '@drizztdourden08/tessera';
import type { BoardBadges } from './tab-badges.type';

const withBadges = (tabs: readonly TabItem[], badges: BoardBadges): TabItem[] =>
  tabs.map((tab) => ({ ...tab, badge: badges[tab.id]?.count ?? 0, badgeTone: badges[tab.id]?.tone }));

export { withBadges };
