/* @layer renderer-app @kind logic */
import type { MenuGroup } from '@drizztdourden08/tessera';
import type { Account } from '../accounts.type';
import { SLOT_CLEAR } from '../next.constants';
import { accountKey } from './account-key';
import { accountName } from './account-name';

const slotGroups = (accounts: readonly Account[], value: string | null, onPick: (key: string) => void, onClear?: () => void): MenuGroup[] => [
  {
    id: 'accounts',
    items: accounts.map((account) => ({
      id: accountKey(account),
      label: accountName(accounts, account.id),
      disabled: accountKey(account) === value,
      onSelect: () => onPick(accountKey(account)),
    })),
  },
  ...(onClear && value ? [{ id: 'clear', items: [{ id: 'clear', label: SLOT_CLEAR, onSelect: onClear }] }] : []),
];

export { slotGroups };
