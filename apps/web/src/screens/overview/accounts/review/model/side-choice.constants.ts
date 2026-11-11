/* @layer renderer-app @kind constants */
import type { SideRole } from './side-choice.type';

const SIDE_ROLES: Readonly<Record<string, SideRole>> = {
  take: 'winner',
  folder_of: 'winner',
  copy_to: 'loser',
  create_on: 'loser',
  delete_on: 'loser',
};

export { SIDE_ROLES };
