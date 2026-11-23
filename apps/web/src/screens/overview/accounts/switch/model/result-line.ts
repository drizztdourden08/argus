/* @layer renderer-app @kind logic */
import type { AccountsSwitch } from '../../../../../data/types';
import type { Account } from '../../accounts.type';
import { accountName, clockText } from '../../model';
import { NO_REASON, NO_RUN_YET } from '../switch.constants';

const sentence = (text: string): string => `${text.trim().replace(/\.$/u, '')}.`;

const outcome = (doc: AccountsSwitch, when: string, to: string): string => {
  if (doc.phase === 'done') return `finished${when}. Desktop is on ${to}`;
  if (doc.phase === 'failed') return `failed${when}: ${doc.error ?? NO_REASON}`;
  if (doc.phase === 'cancelled') return `was cancelled${when}`;
  return `ended${when}`;
};

const resultLine = (doc: AccountsSwitch | null, accounts: readonly Account[]): string => {
  const steps = doc?.steps ?? [];
  if (!doc || steps.length === 0) return NO_RUN_YET;
  const at = clockText(steps.at(-1)?.at);
  const to = accountName(accounts, doc.to);
  return sentence(`Last switch from ${accountName(accounts, doc.from)} to ${to} ${outcome(doc, at ? ` at ${at}` : '', to)}`);
};

export { resultLine };
