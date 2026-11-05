/* @layer renderer-app @kind logic */
import { NO_KIND_LABEL } from './builds-view.constants';
import type { KindBuilds } from './builds-view.type';
import type { LauncherView } from './model.type';

const byKey = (a: KindBuilds, b: KindBuilds): number => {
  if (!a.key !== !b.key) return a.key ? -1 : 1;
  return a.key.localeCompare(b.key, undefined, { numeric: true });
};

const groupByKind = (views: readonly LauncherView[]): KindBuilds[] => {
  const found = new Map<string, LauncherView[]>();
  for (const view of views) {
    const key = view.launcher.kind ?? '';
    found.set(key, [...(found.get(key) ?? []), view]);
  }
  return [...found].map(([key, rows]) => ({ key, label: key || NO_KIND_LABEL, views: rows })).sort(byKey);
};

export { groupByKind };
