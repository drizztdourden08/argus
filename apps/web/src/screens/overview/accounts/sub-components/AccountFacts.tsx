/* @layer renderer-app @kind component */
import { Stack, Text } from '@drizztdourden08/tessera';
import type { AccountFactsProps } from '../accounts.type';

const dayOf = (value: string | number | null | undefined): string => (value ? new Date(value).toLocaleDateString('en-CA') : '');

const AccountFacts = (props: AccountFactsProps) => {
  const { cli, desktopLogin, chats } = props.account;
  const cliText = cli?.loggedIn ? `CLI login valid until ${dayOf(cli.expiresAt)}` : 'CLI not signed in';
  const desktopText = desktopLogin?.saved ? `Desktop login saved ${dayOf(desktopLogin.savedAt)}` : 'Desktop login not saved';
  const chatText = `${chats?.total ?? 0} chats, ${chats?.archived ?? 0} archived, ${chats?.folders ?? 0} folders`;
  return (
    <Stack gap="xs">
      <Text variant="subtitle" size="md">{cliText}</Text>
      <Text variant="subtitle" size="md">{desktopText}</Text>
      <Text variant="subtitle" size="md">{chatText}</Text>
    </Stack>
  );
};

export { AccountFacts };
