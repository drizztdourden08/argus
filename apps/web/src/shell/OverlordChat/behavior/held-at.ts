/* @layer renderer-app @kind logic */
import type { QueueHeld } from '../../../data/overlord';

const heldAt = (held: QueueHeld): string | undefined => (typeof held === 'string' ? undefined : held.at);

export { heldAt };
