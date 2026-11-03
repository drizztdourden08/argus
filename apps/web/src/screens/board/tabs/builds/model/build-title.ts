/* @layer renderer-app @kind logic */
import { TITLE_WORDS } from '../builds.constants';
import type { LauncherDoc } from './model.type';

const firstWords = (cmd: string): string => cmd.trim().split(/\s+/).slice(0, TITLE_WORDS).join(' ');

const buildTitle = (launcher: LauncherDoc): string => launcher.title?.trim() || firstWords(launcher.cmd ?? '') || launcher.name;

export { buildTitle };
