/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import type { DepLink } from './model.type';

const waitsLink = (item: ItemRef): DepLink[] => {
  const wait = item.waitsOn;
  if (!wait) return [];
  return [{ side: 'waits', dep: wait.dep, project: wait.project, item: wait.item, title: wait.title, thread: null, state: wait.state, since: wait.since, reason: wait.reason }];
};

const askedLink = (item: ItemRef): DepLink[] => {
  const asked = item.requestedBy;
  if (!asked) return [];
  return [{ side: 'asked', dep: asked.dep, project: asked.project, item: asked.item, title: asked.title, thread: asked.thread, state: asked.state, since: item.createdAt ?? null, reason: null }];
};

const depLinks = (item: ItemRef): DepLink[] => [...waitsLink(item), ...askedLink(item)];

export { depLinks };
