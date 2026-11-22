/* @layer renderer-app @kind logic */
import type { OverlordStatus } from '../../../data/overlord';
import { STARTING_CHAT } from '../OverlordChat.constants';

const overlordNote = (status: OverlordStatus | null): string => status?.problem ?? STARTING_CHAT;

export { overlordNote };
