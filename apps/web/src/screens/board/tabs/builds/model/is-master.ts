/* @layer renderer-app @kind logic */
import { MASTER } from '../builds.constants';
import type { LauncherDoc } from './model.type';

const isMaster = (launcher: LauncherDoc): boolean => launcher.master ?? launcher.name === MASTER;

export { isMaster };
