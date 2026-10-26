/* @layer renderer-app @kind logic */
import { ID_RANDOM_LENGTH } from './posts.constants';

const newEntryId = (now: number = Date.now(), random: () => number = Math.random): string => {
  const tail = random().toString(36).slice(2, 2 + ID_RANDOM_LENGTH).padEnd(ID_RANDOM_LENGTH, '0');
  return `O-${now.toString(36)}-${tail}`;
};

export { newEntryId };
