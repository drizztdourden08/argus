/* @layer renderer-app @kind logic */
import type { Account } from '../accounts.type';
import { STEP_KIND_LABEL } from '../compare.constants';
import type { SyncStep } from '../compare.type';
import { accountName } from './account-name';
import { clockText } from './clock-text';

const stepLine = (step: SyncStep, accounts: readonly Account[]): string => {
  const kind = step.kind ? STEP_KIND_LABEL[step.kind] : 'Step';
  const what = step.title ?? step.folder ?? step.detail ?? '';
  const where = step.account ? ` (${accountName(accounts, step.account)})` : '';
  return `${clockText(step.at, true)} ${kind}${what ? `: ${what}` : ''}${where}`.trim();
};

export { stepLine };
