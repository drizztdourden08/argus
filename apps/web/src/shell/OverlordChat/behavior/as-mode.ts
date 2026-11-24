/* @layer renderer-app @kind logic */
import type { OverlordMode } from '../../../data/overlord';

const asMode = (mode: string | null): OverlordMode => (mode === 'shadow' || mode === 'live' ? mode : 'off');

export { asMode };
