/* @layer renderer-app @kind types */
import type { ReactNode, RefObject } from 'react';

interface HeaderPopoverProps {
  trigger: ReactNode;
  label: string;
  name?: string;
  hint?: string;
  children: ReactNode;
}

interface PopoverView {
  anchor: RefObject<HTMLButtonElement | null>;
  open: boolean;
  toggle: () => void;
  close: () => void;
}

export type { HeaderPopoverProps, PopoverView };
