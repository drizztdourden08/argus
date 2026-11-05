/* @layer renderer-app @kind logic */
import type { ParsedShellMode } from '../AppShell.type';
import { BOARD_PATH_PATTERN } from './parse-shell-mode.constants';

const decode = (value: string): string => {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
};

const parseShellMode = (pathname: string): ParsedShellMode => {
  const match = BOARD_PATH_PATTERN.exec(pathname);
  if (!match?.[1]) return { mode: 'overview', boardKey: null };
  return { mode: 'boards', boardKey: decode(match[1]) };
};

export { parseShellMode };
