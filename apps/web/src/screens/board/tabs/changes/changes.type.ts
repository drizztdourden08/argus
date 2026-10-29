/* @layer renderer-app @kind types */
import type { WithId } from '../../../../data/react';
import type { CheckRun } from '../../../../data/remote';
import type { PullRequest } from '../../../../data/types';

interface LogTarget {
  runId: string;
  index: number;
  name: string;
  command: string;
}

type PullRequestDoc = WithId<PullRequest>;

interface ChangesTabProps {
  boardKey: string;
}

interface PrListProps {
  boardKey: string;
  prs: readonly PullRequestDoc[];
  onLog: (target: LogTarget) => void;
}

interface PrCardProps {
  boardKey: string;
  pr: PullRequestDoc;
  onLog: (target: LogTarget) => void;
}

interface PrPartProps {
  pr: PullRequestDoc;
}

interface PrChecksProps {
  boardKey: string;
  pr: PullRequestDoc;
  onLog: (target: LogTarget) => void;
}

interface CheckRunViewProps {
  run: CheckRun;
  first: boolean;
  onLog: (target: LogTarget) => void;
}

interface StepListProps {
  run: CheckRun;
  onLog: (target: LogTarget) => void;
}

interface StepLogDrawerProps {
  boardKey: string;
  target: LogTarget | null;
  onClose: () => void;
}

interface StepLogBodyProps {
  boardKey: string;
  target: LogTarget;
}

export type {
  ChangesTabProps, CheckRunViewProps, LogTarget, PrCardProps, PrChecksProps, PrListProps, PrPartProps, PullRequestDoc, StepListProps,
  StepLogBodyProps, StepLogDrawerProps,
};
