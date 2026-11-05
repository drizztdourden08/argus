/* @layer renderer-app @kind logic */
import type { ParsedShellMode } from '../AppShell.type';

const BOARD_PATH_PATTERN = /^\/p\/([^/]+)\/?$/;

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
