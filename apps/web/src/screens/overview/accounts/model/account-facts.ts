/* @layer renderer-app @kind logic */
import type { AccountDoc } from '../../../../data/types';

const cliLine = (cli: AccountDoc['cli']): string => {
  if (!cli?.loggedIn) return 'CLI login needed';
  return cli.loginExpiresAt ? `CLI signed in until ${new Date(cli.loginExpiresAt).toLocaleDateString('en-CA')}` : 'CLI signed in';
};

const desktopLine = (login: AccountDoc['desktopLogin']): string => (login?.saved ? 'Desktop login saved' : 'Desktop login not saved');

const chatLine = (chats: AccountDoc['chats']): string =>
  `${chats?.total ?? 0} chats, ${chats?.archived ?? 0} archived, ${chats?.folders ?? 0} folders`;

const accountFacts = (account: AccountDoc): string =>
  [account.accountId ? `Account ${account.accountId}` : '', cliLine(account.cli), desktopLine(account.desktopLogin), chatLine(account.chats)]
    .filter(Boolean)
    .join('. ');

export { accountFacts };
