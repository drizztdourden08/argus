/* @layer renderer-app @kind types */
interface BoardStageProps {
  boardKey: string;
}

interface BoardReveal {
  ready: boolean;
  reduced: boolean;
  reveal: () => void;
}

export type { BoardReveal, BoardStageProps };
