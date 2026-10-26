/* @layer renderer-app @kind types */
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
  onEscape?: () => void;
}

export type { OwnerBoxProps };
