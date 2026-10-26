/* @layer renderer-app @kind types */
interface OwnerBoxProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  onSubmit: () => void;
  rows?: number;
  disabled?: boolean;
}

export type { OwnerBoxProps };
