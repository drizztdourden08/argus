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
    + `Copy does the same by hand in the direction you pick. Merge makes ${names} match both ways: each gets what it lacks, and archived state follows the newest change.`;
};

export { syncSentence };
