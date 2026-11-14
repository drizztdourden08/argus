/* @layer renderer-app @kind logic */
import type { KeyStatus } from '../../../../../data/overlord';
import { MASK, NO_KEY } from '../OverlordKeySection.constants';

const keyLine = (status: KeyStatus | null): string => (status?.set && status.last4 ? `${MASK}${status.last4}` : NO_KEY);

export { keyLine };
