/* @layer renderer-app @kind logic */
import type { Entry, ItemRef } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { askKind } from './ask-kind';
import { byCreated } from './by-created';
import { settledIds } from './settled-ids';
import type { AskKind, AskLink, EntryDocs } from './model.type';

interface Asking {
  entry: WithId<Entry>;
  kind: AskKind;
}

const isLive = (entry: WithId<Entry>): boolean => !entry.deletedAt && !entry.archivedAt;

const askingEntries = (entries: EntryDocs): Asking[] => {
  const live = entries.filter(isLive);
  const settled = settledIds(live);
  return [...live].sort(byCreated).flatMap((entry) => {
    const kind = askKind(entry, settled);
    return kind ? [{ entry, kind }] : [];
  });
};

const addLink = (links: Map<string, AskLink[]>, itemId: string, link: AskLink): void => {
  const list = links.get(itemId) ?? [];
  if (!list.some((known) => known.id === link.id)) links.set(itemId, [...list, link]);
};

const askLinks = (items: readonly ItemRef[], entries: EntryDocs): Map<string, AskLink[]> => {
  const asking = askingEntries(entries);
  const kinds = new Map(asking.map(({ entry, kind }) => [entry.id, kind]));
  const links = new Map<string, AskLink[]>();
  for (const item of items) {
    const id = item.blocker?.trim() ?? '';
    const kind = kinds.get(id);
    if (kind) addLink(links, item.id, { id, kind, via: 'blocker' });
  }
  for (const { entry, kind } of asking) if (entry.item) addLink(links, entry.item, { id: entry.id, kind, via: 'item' });
  return links;
};

export { askLinks };
