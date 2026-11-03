/* @layer renderer-app @kind logic */
import { kindOf } from '../../../../../data/registry';
import { shownCode } from '../../../../../shell/records';
import { BUILD_KIND } from '../builds.constants';

const buildCode = (name: string): string => {
  const kind = kindOf(BUILD_KIND);
  return kind ? shownCode(kind, name) : `${BUILD_KIND}-${name}`;
};

export { buildCode };
