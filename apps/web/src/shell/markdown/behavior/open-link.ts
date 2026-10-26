/* @layer renderer-app @kind logic */
import { ROUTED_PATH } from '../markdown.constants';
import { leavesFolder } from './leaves-folder';

const openLink = (href: string, go: (path: string) => void): void => {
  if (ROUTED_PATH.test(href)) go(href);
  else if (!leavesFolder(href)) window.open(href, '_blank', 'noopener,noreferrer');
};

export { openLink };
