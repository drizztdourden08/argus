/* @layer renderer-app @kind logic */
import { MASTER } from '../builds.constants';
import type { LauncherDoc } from './model.type';

const pinRank = (launcher: LauncherDoc): number => {
  if (launcher.name === MASTER) return 0;
  return launcher.current ? 1 : 2;
};

const byName = (a: LauncherDoc, b: LauncherDoc): number => a.name.localeCompare(b.name, undefined, { numeric: true });

const sortLaunchers = (docs: readonly LauncherDoc[]): LauncherDoc[] =>
  [...docs].sort((a, b) => pinRank(a) - pinRank(b) || byName(a, b));

export { sortLaunchers };
