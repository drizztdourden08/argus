/* @layer renderer-app @kind logic */
import type { Option } from '../../../../screens/board/shared';
import { threadChoices } from '../../../../screens/board/shared';
import type { Threads } from '../../../../screens/board/tabs/kanban/kanban.type';
import { OWNER, OWNER_TITLE } from '../../../../screens/board/tabs/kanban/model';

const peopleOptions = (threads: Threads, current?: string): Option[] => [{ value: OWNER, label: OWNER_TITLE }, ...threadChoices(threads, current)];

export { peopleOptions };
