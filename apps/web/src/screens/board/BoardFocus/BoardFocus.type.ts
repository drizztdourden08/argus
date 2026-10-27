/* @layer renderer-app @kind types */
interface BoardFocusProps {
  boardKey: string;
}

interface FocusView {
  focus: { focus: string; setFocus: (thread: string) => void };
  showPaused: { on: boolean; set: (on: boolean) => void };
  choices: { options: { value: string; label: string }[]; pausedCount: number };
}

export type { BoardFocusProps, FocusView };
