/* @layer renderer-app @kind logic */
import { depRef, isWaiting } from '../../../../../data/deps';
import type { DepLink } from './model.type';

const depTagLabel = (link: DepLink): string => {
  const ref = depRef(link);
  if (link.state === 'cancelled') return 'request cancelled';
  if (link.side === 'asked') return `for ${ref}`;
  return isWaiting(link.state) ? `waiting on ${ref}` : `${ref} ${link.state}`;
};

export { depTagLabel };
