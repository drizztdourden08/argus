/* @layer renderer-app @kind logic */
import type { CacheUser } from '../../../../../data/cache';

const usersText = (users: readonly CacheUser[]): string => users.map((user) => user.label).join(', ');

export { usersText };
