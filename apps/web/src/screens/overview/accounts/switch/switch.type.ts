/* @layer renderer-app @kind types */
import type { StepperStep } from '@drizztdourden08/tessera';
import type { AccountsSwitch, SwitchStage, SwitchStepState, SwitchSyncDoc, SwitchThreadDoc } from '../../../../data/types';
import type { Account } from '../accounts.type';

type TitleText = (from: string, to: string) => string;

interface SwitchActions {
  busy: boolean;
  start: (to: string, handover: boolean) => void;
  anyway: () => void;
  cancel: () => void;
  skipStep: (stage: SwitchStage) => void;
  skipThread: (board: string, thread: string) => void;
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

interface StepBodyProps {
  doc: AccountsSwitch;
  stage: SwitchStage;
}

interface WizardModel {
  steps: StepperStep[];
  currentId: SwitchStage | undefined;
  complete: boolean;
}

interface WizardView {
  shown: SwitchStage | undefined;
  show: (id: string) => void;
}

interface ThreadListProps {
  threads: readonly SwitchThreadDoc[];
  checked: boolean;
  skippable: boolean;
}

interface ThreadRowProps {
  thread: SwitchThreadDoc;
  skippable: boolean;
}

interface WizardPick {
  stage: SwitchStage;
  from: SwitchStage | undefined;
}

interface SyncBlockProps {
  sync: SwitchSyncDoc | null | undefined;
}

interface SwitchConfirmProps {
  target: Account;
  from: string | null;
  onClose: () => void;
}

export type {
  ProcessDocProps, ProcessPanelProps, ProcessSummaryProps, StageView, StepBodyProps, SwitchActions, SwitchConfirmProps, SwitchLink, SyncBlockProps,
  ThreadListProps, ThreadRowProps, TitleText, WizardModel, WizardPick, WizardView,
};
