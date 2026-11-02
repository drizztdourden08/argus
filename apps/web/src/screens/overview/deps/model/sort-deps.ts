/* @layer renderer-app @kind logic */
import { isLive } from '../../../../data/deps';
import type { DepDoc } from '../../../../data/deps';
import { waitMs } from './wait-ms';

const rank = (doc: DepDoc): number => (isLive(doc.state) ? 0 : 1);

const sortDeps = <T extends DepDoc>(docs: readonly T[], now: number): T[] =>
  [...docs].sort((a, b) => rank(a) - rank(b) || waitMs(b, now) - waitMs(a, now) || a.id.localeCompare(b.id));

export { sortDeps };
