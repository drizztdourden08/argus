/* @layer renderer-app @kind types */
import type { IconName, SignalTone } from '@drizztdourden08/tessera';
import type { WithId } from '../../../../data/react';
import type { CheckRun } from '../../../../data/remote';
import type { PullRequest } from '../../../../data/types';
import type { GroupThreads } from '../../item-groups';

interface LogTarget {
  runId: string;
  index: number;
  name: string;
  command: string;
}

type PullRequestDoc = WithId<PullRequest>;

interface ChecksProgress {
  passed: number;
  total: number;
  failed: boolean;
}

interface PrSignal {
  id: string;
  icon: IconName;
  label: string;
  tone: SignalTone;
}

interface ChangesTabProps {
  boardKey: string;
}

interface PrGroupsProps {
  boardKey: string;
  prs: readonly PullRequestDoc[];
}

interface PrCardProps {
  boardKey: string;
  pr: PullRequestDoc;
  threads: GroupThreads;
}

interface PrFootProps {
  pr: PullRequestDoc;
  runs: readonly CheckRun[] | null;
}

interface PrSignalsProps {
  pr: PullRequestDoc;
  runs: readonly CheckRun[] | null;
}

interface StepListProps {
  run: CheckRun;
  onLog: (target: LogTarget) => void;
}

interface StepLogBodyProps {
  boardKey: string;
  target: LogTarget;
}

export type {
  ChangesTabProps, ChecksProgress, LogTarget, PrCardProps, PrFootProps, PrGroupsProps, PrSignal, PrSignalsProps, PullRequestDoc, StepListProps, StepLogBodyProps,
};
