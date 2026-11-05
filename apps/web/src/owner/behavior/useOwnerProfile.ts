/* @layer renderer-app @kind logic */
import { PORTFOLIO_SCOPE, useOptionalDoc } from '../../data/react';
import type { BoardMeta } from '../../data/types';
import { OWNER_DEFAULTS } from '../owner.constants';
import type { OwnerProfile } from '../owner.type';

const filled = (value: string | undefined, fallback: string): string => (value === undefined || value === '' ? fallback : value);

const useOwnerProfile = (): OwnerProfile => {
  const owner = useOptionalDoc<BoardMeta>(PORTFOLIO_SCOPE, 'meta', 'board')?.owner;
  return {
    name: filled(owner?.name, OWNER_DEFAULTS.name),
    initials: filled(owner?.initials, OWNER_DEFAULTS.initials),
    avatar: owner?.avatar ?? OWNER_DEFAULTS.avatar,
  };
};

export { useOwnerProfile };
