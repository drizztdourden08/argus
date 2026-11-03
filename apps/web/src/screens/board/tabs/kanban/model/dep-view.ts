/* @layer renderer-app @kind logic */
import { isLive } from '../../../../../data/deps';
import type { DepDoc } from '../../../../../data/deps';
import type { DepLink, DepView } from './model.type';

const threadOf = (link: DepLink, doc: DepDoc | null): string | null => {
  if (link.side === 'waits') return doc?.to.thread ?? null;
  return link.thread ?? doc?.from.thread ?? null;
};

const depView = (link: DepLink, doc: DepDoc | null): DepView => {
  const state = doc?.state ?? link.state;
  return {
    ...link,
    state,
    thread: threadOf(link, doc),
    since: doc?.openedAt ?? link.since,
    reason: doc?.reason ?? link.reason,
    history: doc?.history ?? [],
    cancellable: isLive(state),
  };
};

export { depView };
