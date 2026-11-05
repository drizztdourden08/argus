/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { itemCode } from '../../../../../shell';
import type { TabField } from '../../../BoardTabsRow';
import type { Option } from '../../../shared';
import { STAGE_CHOICES } from './builds-view.constants';
import type { BuildFilterKey, BuildFilters } from './builds-view.type';
import type { LauncherView, LauncherViews } from './model.type';

const ITEM_LABEL_CHARS = 40;

const multi = (key: BuildFilterKey, label: string, options: readonly Option[]): TabField<BuildFilters> => ({
  kind: 'multi',
  path: key,
  label,
  options,
  read: (filters) => filters[key],
  write: (values) => ({ [key]: values }) as Partial<BuildFilters>,
});

const everyView = (views: LauncherViews): LauncherView[] => [...views.live, ...views.retired];

const kindChoices = (views: LauncherViews): Option[] =>
  [...new Set(everyView(views).map((view) => view.launcher.kind ?? '').filter(Boolean))].sort().map((kind) => ({ value: kind, label: kind }));

const itemChoices = (views: LauncherViews, items: readonly ItemRef[]): Option[] => {
  const used = new Set(everyView(views).map((view) => view.launcher.for ?? ''));
  return items.filter((item) => used.has(item.id)).map((item) => ({ value: item.id, label: `${itemCode(item)} ${String(item.title).slice(0, ITEM_LABEL_CHARS)}` }));
};

const buildFields = (views: LauncherViews, items: readonly ItemRef[]): TabField<BuildFilters>[] => [
  multi('item', 'Item', itemChoices(views, items)),
  multi('kind', 'Kind', kindChoices(views)),
  multi('stage', 'Stage', STAGE_CHOICES),
];

export { buildFields };
