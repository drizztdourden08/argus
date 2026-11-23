/* @layer renderer-app @kind logic */
import type { AccountsSwitch } from '../../../../../data/types';
import { AT_ZERO_HANDOVER, AT_ZERO_SYNC } from '../switch.constants';

const atZeroText = (doc: AccountsSwitch): string =>
  (doc.threads ?? []).some((thread) => thread.state !== 'skipped') ? AT_ZERO_HANDOVER : AT_ZERO_SYNC;

export { atZeroText };
