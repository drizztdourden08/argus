/* @layer renderer-app @kind logic */
import { toMillis } from '../../format';
import type { WithId } from '../react';
import type { StallDoc } from './stalls.type';

const stallsOf = (docs: readonly WithId<StallDoc>[], project: string | null = null): WithId<StallDoc>[] =>
  docs
    .filter((doc) => project === null || doc.project === project)
    .sort((a, b) => toMillis(a.since) - toMillis(b.since));

export { stallsOf };
