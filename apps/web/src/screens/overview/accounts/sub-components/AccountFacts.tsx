/* @layer renderer-app @kind component */
import { Stack, Text } from '@drizztdourden08/tessera';
import type { AccountDoc } from '../../../../data/types';
import type { AccountFactsProps } from '../accounts.type';

const dayOf = (value: string | number | null | undefined): string => (value ? new Date(value).toLocaleDateString('en-CA') : '');

const shortDayOf = (value: string): string => new Date(value).toLocaleDateString('en-US', { month: 'short', day: 'numeric' });

const cliLine = (cli: AccountDoc['cli']): string => {
  if (!cli?.loggedIn) return 'CLI login needed';
  return cli.loginExpiresAt ? `CLI signed in until ${shortDayOf(cli.loginExpiresAt)}` : 'CLI signed in';
};

const desktopLine = (login: AccountDoc['desktopLogin']): string =>
  login?.saved ? `Desktop login saved ${dayOf(login.savedAt)}` : 'Desktop login not saved';

const chatLine = (chats: AccountDoc['chats']): string =>
  `${chats?.total ?? 0} chats, ${chats?.archived ?? 0} archived, ${chats?.folders ?? 0} folders`;

const AccountFacts = (props: AccountFactsProps) => {
  const { cli, desktopLogin, chats } = props.account;
  return (
    <Stack gap="xs">
      <Text variant="subtitle" size="md">{cliLine(cli)}</Text>
      <Text variant="subtitle" size="md">{desktopLine(desktopLogin)}</Text>
      <Text variant="subtitle" size="md">{chatLine(chats)}</Text>
    </Stack>
  );
};

export { AccountFacts };
