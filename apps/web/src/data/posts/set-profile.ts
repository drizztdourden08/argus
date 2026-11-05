/* @layer renderer-app @kind logic */
import { setDoc } from '../api';
import { PORTFOLIO_SCOPE } from '../react';
import { ownerEntry } from './owner-entry';
import type { ProfileFields } from './posts.type';

const REFUSED_DEFAULT = 'The profile was refused.';

const setProfile = async (fields: ProfileFields): Promise<string | null> => {
  const entry = ownerEntry({ kind: 'profile', ...fields });
  const result = await setDoc(PORTFOLIO_SCOPE, 'entries', String(entry.id), entry);
  return result.status === 'refused' ? (result.outcome ?? REFUSED_DEFAULT) : null;
};

export { setProfile };
