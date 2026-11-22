/* @layer renderer-app @kind logic */
import type { QueueHeld } from '../../../data/overlord';

const heldText = (held: QueueHeld): string => (typeof held === 'string' ? held : held.text);

const heldAt = (held: QueueHeld): string | undefined => (typeof held === 'string' ? undefined : held.at);

export { heldAt, heldText };
