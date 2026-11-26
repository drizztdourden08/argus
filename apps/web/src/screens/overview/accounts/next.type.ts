/* @layer renderer-app @kind types */
import type { AccountsSettingsApi } from '../../../data/accounts';
import type { AccountsStatus, AccountsSwitch, NextRunPhase, NextWhen } from '../../../data/types';
import type { Account, AccountActions } from './accounts.type';
import type { CompareDoc } from './compare.type';

type Direction = 'ltr' | 'rtl' | 'both';

type SlotSide = 'left' | 'right';

type SlotPair = Readonly<Record<SlotSide, string | null>>;

interface NextActions {
  busy: boolean;
  save: (pair: SlotPair) => void;
}

interface NextFeed {
  status: AccountsStatus | null;
  reload: () => void;
}

interface AccountSlotProps {
  label: string;
  accounts: readonly Account[];
  value: string | null;
  hint?: string;
  disabled?: boolean;
  onPick: (key: string) => void;
  onClear: () => void;
}

interface NextStripProps {
  accounts: readonly Account[];
  status: AccountsStatus | null;
  actions: NextActions;
}

interface NextRunProps {
  phase: NextRunPhase | undefined;
}

interface NextArrowProps extends NextRunProps {
  when: NextWhen | undefined;
}

interface SwitchTabProps extends NextStripProps {
  doc: AccountsSwitch | null;
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

interface SyncArrowProps {
  direction: Direction;
  ready: boolean;
  disabled: boolean;
  onDirection: (direction: Direction) => void;
  onGo: () => void;
}

export type {
  AccountCardProps, AccountCardsProps, AccountPaneProps, AccountSlotProps, Direction, NextActions, NextArrowProps, NextFeed, NextRunProps,
  NextStripProps, SlotPair, SlotSide, SwitchTabProps, SyncArrowProps, SyncChoicesProps,
};
