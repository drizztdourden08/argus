/* @layer renderer-app @kind logic */
import { BOARD_HREF } from '../records.constants';
import { parseRecord } from './parse-record';

const entryOfHref = (href: string, boardKey: string): string | null => {
  const found = BOARD_HREF.exec(href);
  if (!found || decodeURIComponent(found[1] ?? '') !== boardKey) return null;
  const ref = parseRecord(found[2] ?? '');
  return ref?.kind === 'entry' && ref.board === null ? ref.id : null;
};

export { entryOfHref };
