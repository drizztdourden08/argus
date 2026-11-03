/* @layer renderer-app @kind logic */
import { kindOf } from '../../../data/registry';
import { shownCode } from '../../../shell/records';
import { RELEASE_KIND } from '../releases.constants';

const releaseCode = (project: string, version: string, prefixed: boolean): string => {
  const kind = kindOf(RELEASE_KIND);
  const code = kind ? shownCode(kind, version) : `V-${version}`;
  return prefixed ? `${project}/${code}` : code;
};

export { releaseCode };
