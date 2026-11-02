/* @layer renderer-app @kind logic */
import type { DepDoc } from '../../../../data/deps';

const changedAt = (doc: DepDoc): string | null => doc.updatedAt ?? doc.history.at(-1)?.at ?? null;

export { changedAt };
