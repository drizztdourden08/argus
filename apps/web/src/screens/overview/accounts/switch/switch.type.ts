/* @layer renderer-app @kind types */
import type { AccountsSwitch, SwitchStage, SwitchStepState, SwitchSyncDoc, SwitchThreadDoc } from '../../../../data/types';
import type { Account } from '../accounts.type';

interface SwitchActions {
  busy: boolean;
  start: (to: string, handover: boolean) => void;
  anyway: () => void;
  cancel: () => void;
}

interface SwitchLink {
  target: string | null;
  clear: () => void;
}

interface StageView {
  stage: SwitchStage;
  label: string;
  state: SwitchStepState;
  text: string;
  at: string;
  detail: string;
}

interface ProcessPanelProps {
  doc: AccountsSwitch | null;
  accounts: readonly Account[];
}

interface ProcessDocProps {
  doc: AccountsSwitch;
}

interface ProcessSummaryProps {
  doc: AccountsSwitch;
  accounts: readonly Account[];
}

interface StageRowProps {
  view: StageView;
  doc: AccountsSwitch;
}

interface ThreadListProps {
  threads: readonly SwitchThreadDoc[];
  checked: boolean;
}

interface SyncBlockProps {
  sync: SwitchSyncDoc | null | undefined;
}

interface SwitchConfirmProps {
  target: Account;
  onClose: () => void;
}

export type {
  ProcessDocProps, ProcessPanelProps, ProcessSummaryProps, StageRowProps, StageView, SwitchActions, SwitchConfirmProps, SwitchLink, SyncBlockProps,
  ThreadListProps,
};
