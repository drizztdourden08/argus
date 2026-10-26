/* @layer renderer-app @kind types */
import type { WithId } from '../../../data/react';
import type { RunEntry } from '../../../data/runs';
import type { LaunchDef } from '../../../data/types';

type RunDef = WithId<LaunchDef>;

interface RunAction {
  busy: boolean;
  act: (action: () => Promise<string>) => Promise<void>;
}

interface Play {
  busy: boolean;
  start: (def: RunDef, label: string) => Promise<void>;
  stop: (run: RunEntry) => Promise<void>;
}

interface RunPlayProps {
  label: string;
  def: RunDef | null;
  hint: string;
  boardKey: string;
  runs: RunEntry[];
}

interface RunStatusProps {
  run: RunEntry;
}

interface PlayDisabledProps {
  label: string;
  hint: string;
}

export type { Play, PlayDisabledProps, RunAction, RunDef, RunPlayProps, RunStatusProps };
