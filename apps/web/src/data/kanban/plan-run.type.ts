/* @layer renderer-app @kind types */
type RunStatus = 'pass' | 'fail' | 'partial' | 'blocked' | 'skip';

type PlanRunAction =
  | { do: 'start'; build?: string }
  | { do: 'record'; test: string; status: RunStatus; note?: string }
  | { do: 'end' };

export type { PlanRunAction, RunStatus };
