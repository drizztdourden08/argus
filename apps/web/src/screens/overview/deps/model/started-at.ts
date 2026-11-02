/* @layer renderer-app @kind logic */
import type { DepDoc } from '../../../../data/deps';

const startedAt = (doc: DepDoc): string | null => doc.openedAt ?? doc.createdAt ?? null;

export { startedAt };
