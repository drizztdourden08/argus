/* @layer renderer-app @kind types */
type ShellMode = 'overview' | 'boards';

interface ParsedShellMode {
  mode: ShellMode;
  boardKey: string | null;
}

interface ShellModeView extends ParsedShellMode {
  boardsAvailable: boolean;
  select: (mode: ShellMode) => void;
}

export type { ParsedShellMode, ShellMode, ShellModeView };
