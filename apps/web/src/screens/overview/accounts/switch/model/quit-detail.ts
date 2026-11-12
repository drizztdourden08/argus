/* @layer renderer-app @kind logic */
import type { AccountsSwitch } from '../../../../../data/types';
import { QUIT_TEXT, WAITING_PHASE } from '../switch.constants';

const quitDetail = (doc: AccountsSwitch | null): string | null => {
  if (doc?.phase !== WAITING_PHASE) return null;
  const detail = doc.steps?.find((step) => step.id === 'close')?.detail;
  return detail ?? QUIT_TEXT;
};

export { quitDetail };
