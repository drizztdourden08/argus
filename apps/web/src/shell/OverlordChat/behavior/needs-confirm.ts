/* @layer renderer-app @kind logic */
import type { OverlordMode } from '../../../data/overlord';

const needsConfirm = (from: OverlordMode, to: OverlordMode) => from === 'live' || to === 'live';

export { needsConfirm };
