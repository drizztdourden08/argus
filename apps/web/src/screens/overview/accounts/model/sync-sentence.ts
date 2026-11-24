/* @layer renderer-app @kind logic */
import type { Account } from '../accounts.type';
import { accountName } from './account-name';

const joinNames = (names: readonly string[]): string => {
  if (names.length < 2) return names.join('');
  return `${names.slice(0, -1).join(', ')} and ${names.at(-1)}`;
};

const syncSentence = (accounts: readonly Account[]): string => {
  const names = joinNames(accounts.map((account) => accountName(accounts, account.id)));
  return `Switching accounts copies the current account's chats, folders and archived state onto the next one, which keeps what only it has. `
    + `Pick two accounts and a direction to copy by hand. Both opens a review of ${names}, where you pick each difference.`;
};

export { syncSentence };
