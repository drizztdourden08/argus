/* @layer renderer-app @kind logic */
import type { QueueHeld } from '../../../data/overlord';

const heldText = (held: QueueHeld): string => (typeof held === 'string' ? held : held.text);

export { heldText };
