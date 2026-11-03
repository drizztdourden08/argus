/* @layer renderer-app @kind logic */
import type { RequestedBy, WaitsOn } from '../../../../../data/deps';
import type { ItemRef } from '../../../../../data/kanban';
import { asList } from './as-list';
import type { DepLink } from './model.type';

const waitsLink = (wait: WaitsOn): DepLink => ({
  side: 'waits', dep: wait.dep, project: wait.project, item: wait.item, title: wait.title, thread: null, state: wait.state, since: wait.since, reason: wait.reason,
});

const askedLink = (asked: RequestedBy, item: ItemRef): DepLink => ({
  side: 'asked', dep: asked.dep, project: asked.project, item: asked.item, title: asked.title, thread: asked.thread, state: asked.state, since: item.createdAt ?? null, reason: null,
});

const depLinks = (item: ItemRef): DepLink[] => [
  ...asList(item.waitsOn).map(waitsLink),
  ...asList(item.requestedBy).map((asked) => askedLink(asked, item)),
];

export { depLinks };
