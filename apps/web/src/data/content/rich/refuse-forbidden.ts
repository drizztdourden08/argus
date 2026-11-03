/* @layer renderer-app @kind logic */
import { RICH_FORBIDDEN } from '../content.constants';
import type { Rejected } from '../content.type';
import { reject } from '../reject';

const refuseForbidden = (field: string, raw: readonly string[]): Rejected | null => {
  const banned = RICH_FORBIDDEN.find((syntax) => raw.some((line) => syntax.pattern.test(line)));
  return banned ? reject('rich-forbidden', `${field} cannot hold a ${banned.name}. Use bullets (- point) and images.`) : null;
};

export { refuseForbidden };
