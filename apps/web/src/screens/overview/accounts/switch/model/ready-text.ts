/* @layer renderer-app @kind logic */
import type { SwitchThreadDoc } from '../../../../../data/types';

const readyText = (threads: readonly SwitchThreadDoc[]): string => {
  const ready = threads.filter((thread) => thread.state === 'ready').length;
  return `${ready} of ${threads.length} ready`;
};

export { readyText };
