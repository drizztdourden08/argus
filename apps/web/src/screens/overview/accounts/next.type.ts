/* @layer renderer-app @kind types */
import type { AccountsSettingsApi } from '../../../data/accounts';
import type { AccountsStatus, AccountsSwitch } from '../../../data/types';
import type { Account, AccountActions } from './accounts.type';
import type { CompareDoc } from './compare.type';

type Direction = 'ltr' | 'rtl' | 'both';

interface NextActions {
  busy: boolean;
  choose: (from: string | null, to: string | null) => void;
  clear: () => void;
}

interface NextFeed {
  status: AccountsStatus | null;
  reload: () => void;
}

interface AccountSlotProps {
  label: string;
  accounts: readonly Account[];
  value: string | null;
  disabled?: boolean;
  onPick: (key: string) => void;
  onClear?: () => void;
}

interface NextStripProps {
  accounts: readonly Account[];
  status: AccountsStatus | null;
  actions: NextActions;
}

interface SwitchTabProps extends NextStripProps {
  doc: AccountsSwitch | null;
  onOpen: (key: string) => void;
}

interface AccountCardsProps {
  accounts: readonly Account[];
  onOpen: (key: string) => void;
}

interface AccountCardProps {
  account: Account;
  onOpen: (key: string) => void;
}

interface AccountPaneProps {
  account: Account;
  accounts: readonly Account[];
  rules: AccountsSettingsApi;
  compare: CompareDoc | null;
  confirming: boolean;
  switching: boolean;
  onSwitch: (key: string) => void;
  onCloseConfirm: () => void;
  onClose: () => void;
}

interface SyncChoicesProps {
  accounts: readonly Account[];
  disabled: boolean;
  actions: AccountActions;
  onReview: () => void;
}

export type { AccountCardProps, AccountCardsProps, AccountPaneProps, AccountSlotProps, Direction, NextActions, NextFeed, NextStripProps, SwitchTabProps, SyncChoicesProps };
