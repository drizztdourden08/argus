/* @layer renderer-app @kind logic */
import { allOpen } from './all-open';

const toggleAll = (open: ReadonlySet<string>, ids: readonly string[]): ReadonlySet<string> => (allOpen(open, ids) ? new Set<string>() : new Set(ids));

export { toggleAll };
