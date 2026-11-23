/* @layer renderer-app @kind logic */
import type { AccountDoc } from '../../../../data/types';
import { LOGIN_DATE } from '../accounts.constants';
import { clockText } from './clock-text';

const cliText = (cli: AccountDoc['cli']): string => {
  if (!cli?.loggedIn) return 'Login needed';
  if (!cli.loginExpiresAt) return 'Signed in';
  return `Signed in until ${new Date(cli.loginExpiresAt).toLocaleDateString('en-CA', LOGIN_DATE)}`;
};

const desktopText = (login: AccountDoc['desktopLogin']): string => (login?.saved ? 'Saved' : 'Not saved');

const sidebarText = (chats: AccountDoc['chats']): string =>
  `${chats?.total ?? 0} chats, ${chats?.archived ?? 0} archived, ${chats?.folders ?? 0} folders`;

const readText = (readAt: string | null | undefined): string => clockText(readAt) || 'Not read yet';

export { cliText, desktopText, readText, sidebarText };
