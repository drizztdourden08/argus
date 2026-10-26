/* @layer renderer-app @kind logic */
import { isNewer } from './is-newer';

const isFresh = (at: unknown, freshSince: number): boolean => Number.isFinite(freshSince) && isNewer(at, freshSince);

export { isFresh };
