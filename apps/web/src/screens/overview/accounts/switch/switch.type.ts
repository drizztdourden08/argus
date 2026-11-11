/* @layer renderer-app @kind types */
import type { AccountsCycle, AccountsSwitch, HandoverDoc } from '../../../../data/types';
import type { Account } from '../accounts.type';

interface SwitchActions {
  busy: boolean;
  start: (to: string, handover: boolean) => void;
  anyway: () => void;
  cancel: () => void;
}

interface SwitchFeed {
  doc: AccountsSwitch | null;
  cycle: AccountsCycle | null;
  handovers: HandoverDoc[];
}

interface SwitchDialogProps {
  target: Account;
  accounts: readonly Account[];
  onClose: () => void;
}

interface SwitchFooterProps {
  target: Account;
  feed: SwitchFeed;
  actions: SwitchActions;
  handover: boolean;
  onClose: () => void;
}

interface SwitchBodyProps {
  target: Account;
  accounts: readonly Account[];
  feed: SwitchFeed;
  actions: SwitchActions;
  handover: boolean;
  onHandover: (next: boolean) => void;
}

interface SwitchOutcomeProps {
  feed: SwitchFeed;
}

interface SwitchStepsProps {
  feed: SwitchFeed;
  actions: SwitchActions;
}

interface SwitchLink {
  target: string | null;
  clear: () => void;
}

interface CyclePanelProps {
  account: Account;
}

interface HandoverLineProps {
  doc: HandoverDoc;
}

export type { CyclePanelProps, HandoverLineProps, SwitchActions, SwitchBodyProps, SwitchDialogProps, SwitchFeed, SwitchFooterProps, SwitchLink, SwitchOutcomeProps, SwitchStepsProps };
