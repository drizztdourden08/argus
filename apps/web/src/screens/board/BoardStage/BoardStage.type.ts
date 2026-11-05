/* @layer renderer-app @kind types */
type SwitchPhase = 'idle' | 'leaving' | 'swap' | 'entering';

interface SwitchState {
  shown: string;
  target: string;
  phase: SwitchPhase;
}

type SwitchAction =
  | { type: 'request'; key: string }
  | { type: 'left' }
  | { type: 'revealed' }
  | { type: 'entered' };

interface BoardStageProps {
  boardKey: string;
}

export type { BoardStageProps, SwitchAction, SwitchPhase, SwitchState };
