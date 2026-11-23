/* @layer renderer-app @kind logic */
import type { MenuGroup } from '@drizztdourden08/tessera';
import { recordMenuGroups } from '../../../../shell';
import { copyText } from '../../shared';
import type { CardFacts, FeedActions } from '../OwnerFeed.type';

const removeGroup = (id: string, actions: FeedActions): MenuGroup => ({
  id: 'remove',
  items: [
    { id: 'archive', label: 'Archive', onSelect: () => void actions.soft('archive', 'entries', id) },
    { id: 'delete', label: 'Delete', tone: 'danger', onSelect: () => void actions.soft('delete', 'entries', id) },
  ],
});

const feedMenu = (facts: CardFacts, actions: FeedActions, onOpen: () => void): MenuGroup[] =>
  recordMenuGroups(facts.kind, facts.code, { open: onOpen, copy: () => copyText(facts.code) }, [removeGroup(facts.code, actions)]);

export { feedMenu };
