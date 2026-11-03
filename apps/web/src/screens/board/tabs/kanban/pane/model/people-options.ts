/* @layer renderer-app @kind logic */
import type { Option } from '../../../../shared';
import { threadChoices } from '../../../../shared';
import type { Threads } from '../../kanban.type';
import { OWNER, OWNER_TITLE } from '../../model';

const peopleOptions = (threads: Threads, current?: string): Option[] => [{ value: OWNER, label: OWNER_TITLE }, ...threadChoices(threads, current)];

export { peopleOptions };
