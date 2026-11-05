/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';

interface OwnerBoxProps {
  value: string;
  onChange: (value: string) => void;
  onSubmit: () => void;
  label?: string;
  ariaLabel?: string;
  placeholder?: string;
  problem?: string | null;
  rows?: number;
  disabled?: boolean;
  autoFocus?: boolean;
  compact?: boolean;
  onEscape?: () => void;
  actions?: ReactNode;
  submitLabel?: string;
}

export type { OwnerBoxProps };
