/* @layer renderer-app @kind logic */
import type { MenuGroup, MenuItem } from '@drizztdourden08/tessera';

const LINK_ID = 'link';

const unlinkItem = (onUnlink: () => void): MenuItem => ({ id: 'unlink', label: 'Unlink from its item', onSelect: onUnlink });

const withItemAfterLink = (group: MenuGroup, item: MenuItem): MenuGroup => {
  const at = group.items.findIndex((entry) => entry.id === LINK_ID);
  if (at < 0) return group;
  return { ...group, items: [...group.items.slice(0, at + 1), item, ...group.items.slice(at + 1)] };
};

const withUnlink = (groups: MenuGroup[], onUnlink: (() => void) | undefined): MenuGroup[] => {
  if (!onUnlink) return groups;
  const item = unlinkItem(onUnlink);
  return groups.map((group) => withItemAfterLink(group, item));
};

export { withUnlink };
