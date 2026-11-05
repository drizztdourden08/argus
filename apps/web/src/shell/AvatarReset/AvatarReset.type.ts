/* @layer renderer-app @kind types */
interface AvatarResetProps {
  disabled: boolean;
  problem: string | null;
  onReset: () => void;
}

export type { AvatarResetProps };
