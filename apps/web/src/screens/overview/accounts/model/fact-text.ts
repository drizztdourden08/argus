/* @layer renderer-app @kind logic */
import type { AccountDoc } from '../../../../data/types';
import { LOGIN_DATE } from '../accounts.constants';

const cliText = (cli: AccountDoc['cli']): string => {
  if (!cli?.loggedIn) return 'Login needed';
  if (!cli.loginExpiresAt) return 'Signed in';
  return `Signed in until ${new Date(cli.loginExpiresAt).toLocaleDateString('en-CA', LOGIN_DATE)}`;
};

export { cliText };
