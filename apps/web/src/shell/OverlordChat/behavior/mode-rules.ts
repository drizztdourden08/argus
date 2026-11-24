/* @layer renderer-app @kind logic */
import type { OverlordMode } from '../../../data/overlord';

const asMode = (mode: string | null): OverlordMode => (mode === 'shadow' || mode === 'live' ? mode : 'off');
const needsConfirm = (from: OverlordMode, to: OverlordMode) => from === 'live' || to === 'live';

export { asMode, needsConfirm };
