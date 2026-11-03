/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { CLOSED_STATUSES } from '../../kanban/model';
import { MASTER } from '../builds.constants';
import type { BuildGroups, ItemBuilds, LauncherView, LauncherViews } from './model.type';

const isMaster = (view: LauncherView): boolean => view.launcher.master ?? view.launcher.name === MASTER;

const isOpen = (item: ItemRef): boolean => !CLOSED_STATUSES.includes(item.status) && !item.deletedAt && !item.archivedAt;

const servesOpen = (view: LauncherView, open: ReadonlyMap<string, ItemRef>): boolean =>
  !view.launcher.relink && open.has(view.launcher.for ?? '');

const byRank = (a: ItemBuilds, b: ItemBuilds): number =>
  (a.item.rank ?? 0) - (b.item.rank ?? 0) || a.item.id.localeCompare(b.item.id, undefined, { numeric: true });

const groupByItem = (views: LauncherViews, items: readonly ItemRef[]): BuildGroups => {
  const open = new Map(items.filter(isOpen).map((item) => [item.id, item]));
  const others = views.live.filter((view) => !isMaster(view));
  const linked = others.filter((view) => servesOpen(view, open));
  const groups = [...open.values()]
    .map((item) => ({ item, views: linked.filter((view) => view.launcher.for === item.id) }))
    .filter((group) => group.views.length > 0)
    .sort(byRank);
  return {
    master: views.live.filter(isMaster),
    items: groups,
    needsLink: others.filter((view) => !linked.includes(view)),
    closed: views.retired,
  };
};

export { groupByItem };
