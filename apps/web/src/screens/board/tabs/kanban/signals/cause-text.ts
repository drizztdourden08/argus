/* @layer renderer-app @kind logic */
import type { BlockedCause } from '../../../../../data/types';

const causeText = (cause: BlockedCause): string => {
  if (cause.kind === 'legacy') return cause.label;
  const who = cause.who ? ` (${cause.who})` : '';
  return cause.label && cause.label !== cause.id ? `${cause.id}${who}: ${cause.label}` : `${cause.id}${who}`;
};

export { causeText };
