/* @layer renderer-app @kind logic */
import type { AccountDoc } from '../../../../data/types';

const sidebarText = (chats: AccountDoc['chats']): string =>
  `${chats?.total ?? 0} chats, ${chats?.archived ?? 0} archived, ${chats?.folders ?? 0} folders`;

export { sidebarText };
