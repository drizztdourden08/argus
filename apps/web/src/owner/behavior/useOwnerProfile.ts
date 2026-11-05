/* @layer renderer-app @kind logic */
import { PORTFOLIO_SCOPE, useOptionalDoc } from '../../data/react';
import type { BoardMeta } from '../../data/types';
import { OWNER_DEFAULTS } from '../owner.constants';
import type { OwnerProfile } from '../owner.type';

const useOwnerProfile = (): OwnerProfile => {
  const owner = useOptionalDoc<BoardMeta>(PORTFOLIO_SCOPE, 'meta', 'board')?.owner;
  return {
    name: owner?.name || OWNER_DEFAULTS.name,
    initials: owner?.initials || OWNER_DEFAULTS.initials,
    avatar: owner?.avatar ?? OWNER_DEFAULTS.avatar,
  };
};

export { useOwnerProfile };
