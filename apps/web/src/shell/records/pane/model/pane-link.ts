/* @layer renderer-app @kind logic */
import { recordSearch } from '../../behavior/record-search';

const paneLink = (code: string): string => {
  const url = new URL(window.location.href);
  url.hash = '';
  url.search = recordSearch('', code);
  return url.toString();
};

export { paneLink };
