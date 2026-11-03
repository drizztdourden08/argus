/* @layer renderer-app @kind logic */
import type { OpenOfHref } from '../records.type';
import { BOARD_PATH, OPEN_PARAM, PANE_PARAM } from '../records.constants';

const openOfHref = (href: string, boardKey: string): OpenOfHref | null => {
  const found = BOARD_PATH.exec(href);
  if (!found || decodeURIComponent(found[1] ?? '') !== boardKey) return null;
  const params = new URLSearchParams(found[2]);
  const code = params.get(OPEN_PARAM);
  const tab = params.get(PANE_PARAM);
  if (!code) return null;
  return tab ? { code, tab } : { code };
};

export { openOfHref };
