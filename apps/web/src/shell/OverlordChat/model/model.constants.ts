/* @layer renderer-app @kind constants */
import type { OverlordState } from '../../../data/overlord';

const OWNER_PREFIX = 'Owner (Argus Overlord tab): ';

const OWNER_SPLIT = /\n\n(?=Owner \(Argus Overlord tab\): )/;

const RUNNING_STATES: readonly OverlordState[] = ['idle', 'turn', 'waiting-approval'];

export { OWNER_PREFIX, OWNER_SPLIT, RUNNING_STATES };
