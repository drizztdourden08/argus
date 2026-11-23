/* @layer renderer-app @kind logic */
import type { Account } from '../accounts.type';
import { accountName } from './account-name';

const joinNames = (names: readonly string[]): string => {
  if (names.length < 2) return names.join('');
  return `${names.slice(0, -1).join(', ')} and ${names.at(-1)}`;
};

const syncSentence = (accounts: readonly Account[]): string =>
  `Sync merges ${joinNames(accounts.map((account) => accountName(accounts, account.id)))}: each gets the chats and folders it lacks, and archived state follows the newest change.`;

export { syncSentence };
