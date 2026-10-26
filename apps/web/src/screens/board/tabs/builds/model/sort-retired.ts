/* @layer renderer-app @kind logic */
import type { LauncherDoc } from './model.type';

const sortRetired = (docs: readonly LauncherDoc[]): LauncherDoc[] =>
  [...docs].sort((a, b) => String(b.retiredAt).localeCompare(String(a.retiredAt)));

export { sortRetired };
