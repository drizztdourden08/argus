/* @layer renderer-app @kind types */
interface LevelControlProps {
  title: string;
  toast: boolean;
  sound: boolean;
  onToast: (on: boolean) => void;
  onSound: (on: boolean) => void;
}

export type { LevelControlProps };
