/* @layer renderer-app @kind logic */
import type { FactsPanelGroup } from '@drizztdourden08/tessera';
import type { Account } from '../accounts.type';
import { desktopText } from './desktop-text';
import { cliText } from './fact-text';
import { readText } from './read-text';
import { sidebarText } from './sidebar-text';

const accountFactGroups = (account: Account, differs: string | null): FactsPanelGroup[] => [[
  { label: 'CLI', value: cliText(account.cli) },
  { label: 'Desktop login', value: desktopText(account.desktopLogin) },
  { label: 'Sidebar', value: sidebarText(account.chats) },
  { label: 'Usage read', value: readText(account.readAt) },
  ...(differs ? [{ label: 'Differs from current', value: differs }] : []),
]];

export { accountFactGroups };
