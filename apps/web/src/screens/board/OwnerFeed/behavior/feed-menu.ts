/* @layer renderer-app @kind logic */
import type { MenuGroup } from '@drizztdourden08/tessera';
import { recordMenuGroups } from '../../../../shell';
import type { ActionHandlers } from '../../../../shell';
import type { FeedCard } from '../../feed-model';
import { copyText } from '../../shared';
import type { CardFacts, FeedActions } from '../OwnerFeed.type';

const softGroup = (collection: 'entries' | 'actions', id: string, actions: FeedActions): MenuGroup => ({
  id: 'remove',
  items: [
    { id: 'archive', label: 'Archive', onSelect: () => void actions.soft('archive', collection, id) },
    { id: 'delete', label: 'Delete', tone: 'danger', onSelect: () => void actions.soft('delete', collection, id) },
  ],
});

const feedMenu = (card: FeedCard, facts: CardFacts, actions: FeedActions, onOpen: () => void): MenuGroup[] => {
  const base: ActionHandlers = { open: onOpen, copy: () => copyText(facts.code) };
  if (card.type === 'test' || card.type === 'bundle') return recordMenuGroups(facts.kind, facts.code, base);
  if (card.type === 'action') {
    const action = card.data;
    return recordMenuGroups(facts.kind, facts.code, { ...base, done: () => void actions.done(action) }, [softGroup('actions', facts.code, actions)]);
  }
  return recordMenuGroups(facts.kind, facts.code, base, [softGroup('entries', facts.code, actions)]);
};

export { feedMenu };
