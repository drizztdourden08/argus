/* @layer renderer-app @kind logic */
import { LONG_WAIT_MS, isWaiting } from '../../../../data/deps';
import type { DepDoc } from '../../../../data/deps';
import { waitMs } from './wait-ms';

const needsLook = (doc: DepDoc, now: number): boolean =>
  doc.state === 'blocked' || (isWaiting(doc.state) && waitMs(doc, now) > LONG_WAIT_MS);

const depBadge = (docs: readonly DepDoc[], now: number): number => docs.filter((doc) => needsLook(doc, now)).length;

export { depBadge };
