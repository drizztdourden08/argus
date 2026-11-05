/* @layer renderer-app @kind logic */
import { OWNER_DEFAULTS } from '../../../../owner';
import type { OwnerProfile } from '../../../../owner';
import type { Option } from '../../../../screens/board/shared';
import { threadChoices } from '../../../../screens/board/shared';
import type { Threads } from '../../../../screens/board/tabs/kanban/kanban.type';
import { OWNER } from '../../../../screens/board/tabs/kanban/model';

const ownerLabel = (owner: OwnerProfile): string => (owner.name === OWNER_DEFAULTS.name ? owner.name : `${owner.name} (you)`);

const peopleOptions = (threads: Threads, current?: string, owner: OwnerProfile = OWNER_DEFAULTS): Option[] => [
  { value: OWNER, label: ownerLabel(owner) },
  ...threadChoices(threads, current),
];

export { ownerLabel, peopleOptions };
