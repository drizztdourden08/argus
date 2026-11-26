/* @layer renderer-app @kind logic */
import type { Account } from '../accounts.type';
import { SLOT_AUTO, SLOT_PICKED } from '../next.constants';
import { accountName } from './account-name';

const slotHint = (accounts: readonly Account[], picked: string | null, auto: string | null): string => {
  if (picked) return SLOT_PICKED;
  return auto ? `${SLOT_AUTO}: ${accountName(accounts, auto)}` : SLOT_AUTO;
};

export { slotHint };
