/* @layer renderer-app @kind logic */
import type { AccountsSwitch } from '../../../../../data/types';
import type { Account } from '../../accounts.type';
import { accountName } from '../../model';
import { MOVING_TITLE, PHASE_TITLE } from '../switch.constants';

const processTitle = (doc: AccountsSwitch, accounts: readonly Account[]): string => {
  const title = (doc.phase ? PHASE_TITLE[doc.phase] : undefined) ?? MOVING_TITLE;
  return title(accountName(accounts, doc.from), accountName(accounts, doc.to));
};

export { processTitle };
