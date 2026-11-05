/* @layer renderer-app @kind types */
import type { ShellMode } from '../AppShell/AppShell.type';

interface ShellBarProps {
  mode: ShellMode;
  boardKey: string | null;
  boardsAvailable: boolean;
  onMode: (mode: ShellMode) => void;
}

export type { ShellBarProps };
