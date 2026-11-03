/* @layer renderer-app @kind logic */
import { nonEmpty } from '../../../shared';
import { TITLE_WORDS } from '../builds.constants';
import type { LauncherDoc } from './model.type';

const firstWords = (cmd: string): string => cmd.trim().split(/\s+/).slice(0, TITLE_WORDS).join(' ');

const buildTitle = (launcher: LauncherDoc): string => nonEmpty(launcher.title?.trim(), nonEmpty(firstWords(launcher.cmd ?? ''), launcher.name));

export { buildTitle };
