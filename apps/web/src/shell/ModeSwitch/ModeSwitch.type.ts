/* @layer renderer-app @kind types */
import type { ShellMode } from '../AppShell/AppShell.type';

interface ModeSwitchProps {
  mode: ShellMode;
  boardsAvailable: boolean;
  onChange: (mode: ShellMode) => void;
}

export type { ModeSwitchProps };
