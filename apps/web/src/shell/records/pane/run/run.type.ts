/* @layer renderer-app @kind types */
import type { ButtonVariant, OutcomeStep } from '@drizztdourden08/tessera';
import type { RunStatus } from '../../../../data/kanban';
import type { WithId } from '../../../../data/react';
import type { Test } from '../../../../data/types';
import type { PaneData } from '../pane.type';

type SummaryKey = RunStatus | 'notRun';

interface RunResult {
  test: string;
  status: RunStatus;
  note: string;
  at: string;
}

interface RunSummary {
  pass: number;
  fail: number;
  partial: number;
  blocked: number;
  skip: number;
  notRun: number;
  total: number;
}

interface PlanRun {
  id: string;
  n: number;
  build: string | null;
  by: string;
  startedAt: string;
  endedAt: string | null;
  results: RunResult[];
  summary: string;
}

interface PlanProgress {
  summary: RunSummary;
  live: boolean;
  cases: number;
}

interface RunOutcome {
  status: RunStatus;
  label: string;
  key: 's' | 'b' | 'm' | 'f' | 'p';
  variant: ButtonVariant;
}

interface RunnerState {
  cursor: number;
  note: string;
  asked: RunStatus | null;
}

type RunnerAction =
  | { type: 'goto'; index: number; total: number }
  | { type: 'step'; by: 1 | -1; total: number }
  | { type: 'note'; text: string }
  | { type: 'ask'; status: RunStatus }
  | { type: 'recorded'; total: number };

interface RunOps {
  start: () => Promise<boolean>;
  record: (test: string, status: RunStatus, note: string) => Promise<boolean>;
  end: () => Promise<boolean>;
}

interface RunPanelProps {
  data: PaneData;
}

interface RunBodyProps {
  data: PaneData;
  run: PlanRun | null;
  ids: readonly string[];
  ops: RunOps;
}

interface RunStageProps {
  data: PaneData;
  run: PlanRun;
  ids: readonly string[];
  ops: RunOps;
}

interface RunHeadProps {
  run: PlanRun;
  position: number;
  total: number;
  steps: readonly OutcomeStep[];
  onSelect: (id: string) => void;
}

interface CaseViewProps {
  id: string;
  test: WithId<Test> | undefined;
  open: (id: string) => void;
}

interface CaseLinesProps {
  title: string;
  lines: readonly string[];
  ordered?: boolean;
}

interface CaseLinksProps {
  test: WithId<Test>;
  open: (id: string) => void;
}

interface EndRunProps {
  left: number;
  ops: RunOps;
}

interface NoteFieldProps {
  value: string;
  problem: string | null;
  disabled: boolean;
  onChange: (text: string) => void;
}

interface OutcomeBarProps {
  disabled: boolean;
  first: boolean;
  last: boolean;
  onRecord: (status: RunStatus) => void;
  onStep: (by: 1 | -1) => void;
}

interface RunSummaryProps {
  data: PaneData;
  run: PlanRun;
  ids: readonly string[];
  ops: RunOps;
}

interface RunStartProps {
  ops: RunOps;
  again: boolean;
}

interface RunKeyHandlers {
  onRecord: (status: RunStatus) => void;
  onStep: (by: 1 | -1) => void;
}

export type {
  CaseLinesProps, PlanProgress, RunBodyProps, CaseLinksProps, CaseViewProps, EndRunProps, NoteFieldProps, OutcomeBarProps, PlanRun, RunHeadProps, RunKeyHandlers, RunnerAction, RunnerState, RunOps, RunOutcome, RunPanelProps, RunResult, RunStageProps, RunStartProps,
  RunSummary, RunSummaryProps, SummaryKey,
};
