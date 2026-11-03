/* @layer renderer-app @kind logic */
import type { PaneBadge, PaneData, RecordData } from '../pane.type';

const itemBadge =
  (badge: (data: PaneData) => PaneBadge | undefined) =>
  (data: RecordData): PaneBadge | undefined =>
    data.item ? badge(data.item) : undefined;

export { itemBadge };
